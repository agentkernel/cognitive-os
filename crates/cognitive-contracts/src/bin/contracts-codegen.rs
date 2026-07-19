//! Schema-to-code generator for the CognitiveOS reference implementation
//! (ADR-0006 code generation policy; IMP-08 minimal-core-first ordering).
//!
//! Reads the minimal core object set (whitepaper appendix A.1, 14 objects)
//! plus its `$ref` closure from `specs/schemas/` and emits committed,
//! reviewable language bindings:
//!
//! - Rust modules under `crates/cognitive-contracts/src/generated/`
//! - TypeScript modules under `packages/contracts-ts/src/generated/`
//!
//! Every emitted file carries a generation header with the source schema
//! path, its canonical content digest (domain `schema-bundle/0.1`, matching
//! the schema-bundle manifest per-asset digest), and the generator version.
//! CI regenerates and diffs; a dirty diff fails the build (ADR-0006 item 4).
//!
//! Bindings are SHAPE-LEVEL: conditional (`if`/`then`), `allOf` const
//! refinements, `contains`, and cross-field constraints stay enforced by
//! JSON Schema validation (`tools/src/check-consistency.mjs`, contract
//! tests). The generator fails loudly on constructs outside the supported
//! subset instead of guessing.
//!
//! Regeneration procedure (also in ADR-0006):
//! `cargo run -p cognitive-contracts --bin contracts-codegen && cargo fmt --all`

use cognitive_contracts::canonical;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// Generator version: bump on any output-affecting change (reviewable per
/// ADR-0006 "the generator becomes a governed tool").
const GENERATOR_VERSION: &str = "0.1.0";

/// Minimal core object set (IMP-08 / whitepaper appendix A.1, 14 objects)
/// mapped to their registered machine schemas, plus the support contracts
/// named by ADR-0006 (common-defs, governed-object-header, object-reference).
/// A.1 objects without a same-named document schema map to their closest
/// registered machine surface:
/// - TenantContext        -> governance-domain-context (discriminated union)
/// - AgentExecution       -> agent-execution-binding (identity binding)
/// - Task / TaskContract  -> task-contract (Task itself is a lifecycle
///   state machine, specs/transitions/task.transitions.json)
/// - StateSnapshot        -> world-state (fixed-version world read view)
/// - OperationDescriptor  -> operation-summary (registered descriptor
///   projection; the full descriptor family follows its consuming milestone)
/// - Checkpoint           -> loop-checkpoint (registered checkpoint payload)
/// - ContextRequest / ContextView -> both schemas generated
const CORE_SET: [&str; 17] = [
    "authorization-capability.schema.json",
    "common-defs.schema.json",
    "context-request.schema.json",
    "context-view.schema.json",
    "effect.schema.json",
    "event.schema.json",
    "governance-domain-context.schema.json",
    "governed-object-header.schema.json",
    "intent.schema.json",
    "loop-checkpoint.schema.json",
    "object-reference.schema.json",
    "operation-summary.schema.json",
    "principal.schema.json",
    "resource-scope.schema.json",
    "task-contract.schema.json",
    "world-state.schema.json",
    "agent-execution-binding.schema.json",
];

/// Legacy `$defs` excluded from generation: deprecated, zero-reference,
/// adapter-only shapes (F-003 retention decision; governed-object-contract
/// section 6). Generating bindings for them would reintroduce the dual
/// track at the type level.
const EXCLUDED_DEFS: [(&str, &str); 2] = [
    ("common-defs.schema.json", "metadata"),
    ("common-defs.schema.json", "strongRef"),
];

fn main() {
    if let Err(err) = run() {
        eprintln!("contracts-codegen: {err}");
        std::process::exit(1);
    }
}

type DynError = Box<dyn Error>;

fn run() -> Result<(), DynError> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let schema_dir = repo_root.join("specs").join("schemas");

    // Load the transitive file-level $ref closure of the core set.
    let mut docs: BTreeMap<String, Value> = BTreeMap::new();
    let mut queue: Vec<String> = CORE_SET.iter().map(|s| s.to_string()).collect();
    while let Some(name) = queue.pop() {
        if docs.contains_key(&name) {
            continue;
        }
        let raw =
            fs::read_to_string(schema_dir.join(&name)).map_err(|e| format!("read {name}: {e}"))?;
        let doc: Value = serde_json::from_str(&raw).map_err(|e| format!("parse {name}: {e}"))?;
        for target in file_refs(&doc) {
            if !docs.contains_key(&target) {
                queue.push(target);
            }
        }
        docs.insert(name, doc);
    }

    let mut rust_modules: Vec<(String, String)> = Vec::new();
    let mut ts_modules: Vec<(String, String)> = Vec::new();
    for (file, doc) in &docs {
        let digest = schema_digest(doc)?;
        let module = generate_module(file, doc, &docs)?;
        rust_modules.push((
            module.rust_mod_name.clone(),
            module.render_rust(file, &digest),
        ));
        ts_modules.push((module.ts_file_stem.clone(), module.render_ts(file, &digest)));
    }

    let rust_dir = repo_root
        .join("crates")
        .join("cognitive-contracts")
        .join("src")
        .join("generated");
    let ts_dir = repo_root
        .join("packages")
        .join("contracts-ts")
        .join("src")
        .join("generated");
    fs::create_dir_all(&rust_dir)?;
    fs::create_dir_all(&ts_dir)?;

    let mut written = 0usize;
    for (mod_name, content) in &rust_modules {
        written += write_if_changed(&rust_dir.join(format!("{mod_name}.rs")), content)?;
    }
    written += write_if_changed(&rust_dir.join("mod.rs"), &render_rust_mod_rs(&rust_modules))?;
    for (stem, content) in &ts_modules {
        written += write_if_changed(&ts_dir.join(format!("{stem}.ts")), content)?;
    }
    written += write_if_changed(&ts_dir.join("index.ts"), &render_ts_index(&ts_modules))?;

    println!(
        "contracts-codegen v{GENERATOR_VERSION}: {} schemas -> {} Rust + {} TS modules ({} files rewritten)",
        docs.len(),
        rust_modules.len(),
        ts_modules.len(),
        written
    );
    println!("reminder: run `cargo fmt --all` after regeneration (ADR-0006 pipeline step)");
    Ok(())
}

fn write_if_changed(path: &PathBuf, content: &str) -> Result<usize, DynError> {
    let current = fs::read_to_string(path).unwrap_or_default();
    if current == content {
        return Ok(0);
    }
    fs::write(path, content).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(1)
}

/// Canonical content digest of a schema document under `schema-bundle/0.1`
/// (the per-asset digest of the schema-bundle manifest, section 13 of
/// docs/standards/canonical-encoding-and-digest.md).
fn schema_digest(doc: &Value) -> Result<String, DynError> {
    let bytes = canonical::canonical_bytes_of_value(doc)?;
    Ok(canonical::digest(&bytes, "schema-bundle/0.1")?)
}

/// All file-level `$ref` targets of a document.
fn file_refs(node: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    collect_file_refs(node, &mut out);
    out
}

fn collect_file_refs(node: &Value, out: &mut BTreeSet<String>) {
    match node {
        Value::Object(map) => {
            if let Some(Value::String(r)) = map.get("$ref") {
                let file = r.split('#').next().unwrap_or("");
                if !file.is_empty() {
                    out.insert(file.to_string());
                }
            }
            for value in map.values() {
                collect_file_refs(value, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_file_refs(item, out);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Intermediate representation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Ty {
    String,
    I64,
    Bool,
    /// Arbitrary JSON value (empty schema / unconstrained payload).
    Any,
    /// Object with unconstrained members.
    AnyMap,
    /// Object used as a string-keyed map with typed values.
    Map(Box<Ty>),
    Vec(Box<Ty>),
    /// Reference to a named generated type.
    Named {
        module: String,
        name: String,
    },
}

#[derive(Debug, Clone)]
struct Field {
    json_name: String,
    ty: Ty,
    required: bool,
    nullable: bool,
    doc: Option<String>,
}

#[derive(Debug, Clone)]
enum NamedType {
    Struct {
        name: String,
        doc: Option<String>,
        deny_unknown: bool,
        fields: Vec<Field>,
    },
    /// Closed string enumeration -> Rust enum / TS literal union.
    Enum {
        name: String,
        doc: Option<String>,
        values: Vec<String>,
    },
    /// Untagged union of named alternatives (schema `oneOf`).
    Union {
        name: String,
        doc: Option<String>,
        variants: Vec<Ty>,
    },
    /// String newtype for pattern-constrained identifiers/digests.
    Newtype { name: String, doc: Option<String> },
    /// Plain alias (constraint-only string forms).
    Alias {
        name: String,
        doc: Option<String>,
        ty: Ty,
    },
}

struct Module {
    rust_mod_name: String,
    ts_file_stem: String,
    types: Vec<NamedType>,
}

struct ModuleBuilder<'a> {
    file: &'a str,
    module: String,
    docs: &'a BTreeMap<String, Value>,
    types: Vec<NamedType>,
    names: BTreeSet<String>,
}

fn module_of(file: &str) -> String {
    file.trim_end_matches(".schema.json").replace('-', "_")
}

fn ts_stem_of(file: &str) -> String {
    file.trim_end_matches(".schema.json").to_string()
}

fn pascal(input: &str) -> String {
    let mut out = String::new();
    let mut upper_next = true;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if upper_next {
                out.extend(ch.to_uppercase());
                upper_next = false;
            } else {
                out.push(ch);
            }
        } else {
            upper_next = true;
        }
    }
    out
}

/// PascalCase for enum variants; ALL-CAPS values keep only the first letter
/// capitalized per segment (COMMITTED -> Committed).
fn variant_name(value: &str) -> String {
    let mut out = String::new();
    for segment in value.split(|c: char| !c.is_ascii_alphanumeric()) {
        if segment.is_empty() {
            continue;
        }
        if segment
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            let mut chars = segment.chars();
            if let Some(first) = chars.next() {
                out.push(first);
                out.push_str(&chars.as_str().to_lowercase());
            }
        } else {
            out.push_str(&pascal(segment));
        }
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'V');
    }
    out
}

fn doc_of(schema: &Value) -> Option<String> {
    schema
        .get("description")
        .and_then(Value::as_str)
        .map(|s| s.replace('\n', " "))
}

fn generate_module(
    file: &str,
    doc: &Value,
    docs: &BTreeMap<String, Value>,
) -> Result<Module, DynError> {
    let mut builder = ModuleBuilder {
        file,
        module: module_of(file),
        docs,
        types: Vec::new(),
        names: BTreeSet::new(),
    };

    // Named definitions first ($defs), then the root type.
    if let Some(defs) = doc.get("$defs").and_then(Value::as_object) {
        for (def_key, def_schema) in defs {
            if EXCLUDED_DEFS.contains(&(file, def_key.as_str())) {
                continue;
            }
            let name = pascal(def_key);
            builder.named_type_of(&name, def_schema)?;
        }
    }
    if doc.get("properties").is_some() || doc.get("oneOf").is_some() {
        let root_name = doc
            .get("title")
            .and_then(Value::as_str)
            .map(pascal)
            .ok_or_else(|| format!("{file}: root schema has no title"))?;
        // Defs-only files (common-defs) have a title but no root object.
        if doc.get("properties").is_some() || root_oneof_is_type_union(doc) {
            builder.named_type_of(&root_name, doc)?;
        }
    }

    Ok(Module {
        rust_mod_name: builder.module.clone(),
        ts_file_stem: ts_stem_of(file),
        types: builder.types,
    })
}

/// Root `oneOf` is a type union only when branches are `$ref`s (not
/// required-only constraint branches like event payload/payload_ref).
fn root_oneof_is_type_union(schema: &Value) -> bool {
    schema
        .get("oneOf")
        .and_then(Value::as_array)
        .is_some_and(|branches| branches.iter().all(|b| b.get("$ref").is_some()))
}

impl ModuleBuilder<'_> {
    /// Ensure a named type for `schema` exists in this module; returns its Ty.
    fn named_type_of(&mut self, name: &str, schema: &Value) -> Result<Ty, DynError> {
        let named = Ty::Named {
            module: self.module.clone(),
            name: name.to_string(),
        };
        if self.names.contains(name) {
            return Ok(named);
        }
        self.names.insert(name.to_string());
        let doc = doc_of(schema);

        // String enumerations.
        if let Some(values) = string_enum_values(schema) {
            self.types.push(NamedType::Enum {
                name: name.to_string(),
                doc,
                values,
            });
            return Ok(named);
        }
        // Pattern-constrained string newtypes (digest, uuidV7).
        if schema.get("type").and_then(Value::as_str) == Some("string") {
            if schema.get("pattern").is_some() {
                self.types.push(NamedType::Newtype {
                    name: name.to_string(),
                    doc: doc.or_else(|| {
                        schema
                            .get("pattern")
                            .and_then(Value::as_str)
                            .map(|p| format!("Pattern: `{p}`"))
                    }),
                });
            } else {
                self.types.push(NamedType::Alias {
                    name: name.to_string(),
                    doc,
                    ty: Ty::String,
                });
            }
            return Ok(named);
        }
        // Union of named alternatives.
        if root_oneof_is_type_union(schema) && schema.get("properties").is_none() {
            let branches = schema
                .get("oneOf")
                .and_then(Value::as_array)
                .ok_or("oneOf")?
                .clone();
            let mut variants = Vec::new();
            for branch in &branches {
                let (ty, nullable) = self.type_of(name, "variant", branch)?;
                if nullable {
                    return Err(format!("{}: nullable branch in union {name}", self.file).into());
                }
                variants.push(ty);
            }
            self.types.push(NamedType::Union {
                name: name.to_string(),
                doc,
                variants,
            });
            return Ok(named);
        }
        // Objects with declared properties -> struct.
        if let Some(props) = schema.get("properties").and_then(Value::as_object) {
            let required: BTreeSet<&str> = schema
                .get("required")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let deny_unknown = schema.get("additionalProperties") == Some(&Value::Bool(false));
            let mut fields = Vec::new();
            for (prop, prop_schema) in props {
                let (ty, nullable) = self.type_of(name, prop, prop_schema)?;
                fields.push(Field {
                    json_name: prop.clone(),
                    ty,
                    required: required.contains(prop.as_str()),
                    nullable,
                    doc: doc_of(prop_schema),
                });
            }
            self.types.push(NamedType::Struct {
                name: name.to_string(),
                doc,
                deny_unknown,
                fields,
            });
            return Ok(named);
        }
        Err(format!(
            "{}: unsupported construct for named type {name}: {}",
            self.file,
            serde_json::to_string(schema).unwrap_or_default()
        )
        .into())
    }

    /// Type of a property schema. Returns (type, nullable).
    fn type_of(
        &mut self,
        parent: &str,
        prop: &str,
        schema: &Value,
    ) -> Result<(Ty, bool), DynError> {
        // $ref
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            return Ok((self.resolve_ref(reference)?, false));
        }
        // allOf: [$ref, const refinements / conditionals] -> the $ref target.
        // allOf carrying only conditionals (if/then) refines an inline object
        // and is dropped at the type level (schema-enforced).
        if let Some(all_of) = schema.get("allOf").and_then(Value::as_array) {
            let refs: Vec<&Value> = all_of.iter().filter(|b| b.get("$ref").is_some()).collect();
            match refs.len() {
                1 => {
                    let reference = refs[0]
                        .get("$ref")
                        .and_then(Value::as_str)
                        .ok_or("allOf $ref")?;
                    return Ok((self.resolve_ref(reference)?, false));
                }
                0 => { /* conditionals only: fall through to type dispatch */ }
                _ => {
                    return Err(
                        format!("{}: unsupported allOf at {parent}.{prop}", self.file).into(),
                    );
                }
            }
        }
        // oneOf
        if let Some(branches) = schema.get("oneOf").and_then(Value::as_array) {
            let null_count = branches
                .iter()
                .filter(|b| b.get("type").and_then(Value::as_str) == Some("null"))
                .count();
            let non_null: Vec<&Value> = branches
                .iter()
                .filter(|b| b.get("type").and_then(Value::as_str) != Some("null"))
                .collect();
            if null_count == 1 && non_null.len() == 1 {
                let (ty, _) = self.type_of(parent, prop, non_null[0])?;
                return Ok((ty, true));
            }
            if null_count == 0 && !non_null.is_empty() {
                // Primitive value union (e.g. integer|string pinned versions).
                let name = format!("{parent}{}Value", pascal(prop));
                let mut variants = Vec::new();
                for branch in &non_null {
                    let (ty, _) = self.type_of(parent, prop, branch)?;
                    variants.push(ty);
                }
                if !self.names.contains(&name) {
                    self.names.insert(name.clone());
                    self.types.push(NamedType::Union {
                        name: name.clone(),
                        doc: None,
                        variants,
                    });
                }
                return Ok((
                    Ty::Named {
                        module: self.module.clone(),
                        name,
                    },
                    false,
                ));
            }
            return Err(format!("{}: unsupported oneOf at {parent}.{prop}", self.file).into());
        }
        // enum
        if string_enum_values(schema).is_some() {
            let name = schema
                .get("title")
                .and_then(Value::as_str)
                .map(pascal)
                .unwrap_or_else(|| format!("{parent}{}", pascal(prop)));
            return Ok((self.named_type_of(&name, schema)?, false));
        }
        // const
        if let Some(const_value) = schema.get("const") {
            return match const_value {
                Value::String(s) => {
                    let name = schema
                        .get("title")
                        .and_then(Value::as_str)
                        .map(pascal)
                        .unwrap_or_else(|| format!("{parent}{}", pascal(prop)));
                    let synthetic = serde_json::json!({ "enum": [s] });
                    Ok((self.named_type_of(&name, &synthetic)?, false))
                }
                Value::Bool(_) => Ok((Ty::Bool, false)),
                other => Err(format!(
                    "{}: unsupported const {other} at {parent}.{prop}",
                    self.file
                )
                .into()),
            };
        }
        // Typed primitives and containers.
        match schema.get("type").and_then(Value::as_str) {
            Some("string") => Ok((Ty::String, false)),
            Some("integer") => Ok((Ty::I64, false)),
            Some("boolean") => Ok((Ty::Bool, false)),
            Some("array") => {
                let items = schema.get("items").ok_or_else(|| {
                    format!("{}: array without items at {parent}.{prop}", self.file)
                })?;
                let (item_ty, nullable) = self.type_of(parent, &singular(prop), items)?;
                if nullable {
                    return Err(
                        format!("{}: nullable array item at {parent}.{prop}", self.file).into(),
                    );
                }
                Ok((Ty::Vec(Box::new(item_ty)), false))
            }
            Some("object") => {
                let has_declared_props = schema
                    .get("properties")
                    .and_then(Value::as_object)
                    .is_some_and(|p| !p.is_empty());
                if has_declared_props {
                    let name = schema
                        .get("title")
                        .and_then(Value::as_str)
                        .map(pascal)
                        .unwrap_or_else(|| format!("{parent}{}", pascal(prop)));
                    return Ok((self.named_type_of(&name, schema)?, false));
                }
                match schema.get("additionalProperties") {
                    None | Some(Value::Bool(true)) => Ok((Ty::AnyMap, false)),
                    Some(Value::Bool(false)) => Err(format!(
                        "{}: closed object without properties at {parent}.{prop}",
                        self.file
                    )
                    .into()),
                    Some(ap_schema) => {
                        let (value_ty, _) = self.type_of(parent, prop, ap_schema)?;
                        Ok((Ty::Map(Box::new(value_ty)), false))
                    }
                }
            }
            Some(other) => {
                Err(format!("{}: unsupported type {other} at {parent}.{prop}", self.file).into())
            }
            None => {
                // Empty schema (unconstrained payload).
                if schema.as_object().is_some_and(|o| o.is_empty()) {
                    Ok((Ty::Any, false))
                } else {
                    Err(format!(
                        "{}: unsupported schema at {parent}.{prop}: {}",
                        self.file,
                        serde_json::to_string(schema).unwrap_or_default()
                    )
                    .into())
                }
            }
        }
    }

    fn resolve_ref(&mut self, reference: &str) -> Result<Ty, DynError> {
        let (file_part, pointer) = match reference.split_once('#') {
            Some((f, p)) => (f, p),
            None => (reference, ""),
        };
        let target_file = if file_part.is_empty() {
            self.file.to_string()
        } else {
            file_part.to_string()
        };
        let target_module = module_of(&target_file);
        if pointer.is_empty() {
            // Whole-file reference -> root type name from the target's title.
            let target_doc = self
                .docs
                .get(&target_file)
                .ok_or_else(|| format!("unresolved $ref file {target_file}"))?;
            let root = target_doc
                .get("title")
                .and_then(Value::as_str)
                .map(pascal)
                .ok_or_else(|| format!("{target_file}: no title for root ref"))?;
            return Ok(Ty::Named {
                module: target_module,
                name: root,
            });
        }
        let def_key = pointer
            .strip_prefix("/$defs/")
            .ok_or_else(|| format!("unsupported $ref pointer {reference}"))?;
        if EXCLUDED_DEFS.contains(&(target_file.as_str(), def_key)) {
            return Err(format!(
                "{}: $ref to excluded legacy def {reference} (F-003 dual-track ban)",
                self.file
            )
            .into());
        }
        let name = pascal(def_key);
        if target_file == self.file {
            // Internal def: generated in $defs pass (or on demand).
            let target_doc = self
                .docs
                .get(&target_file)
                .and_then(|d| d.get("$defs"))
                .and_then(|d| d.get(def_key))
                .cloned()
                .ok_or_else(|| format!("unresolved internal $ref {reference}"))?;
            return self.named_type_of(&name, &target_doc);
        }
        Ok(Ty::Named {
            module: target_module,
            name,
        })
    }
}

fn singular(prop: &str) -> String {
    // Only used for synthetic item-type names: refs -> Ref, items -> Item.
    let p = prop.strip_suffix('s').unwrap_or(prop);
    format!("{p}_item")
}

fn string_enum_values(schema: &Value) -> Option<Vec<String>> {
    let values = schema.get("enum")?.as_array()?;
    let strings: Vec<String> = values
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect();
    if strings.len() == values.len() && !strings.is_empty() {
        Some(strings)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Rust rendering
// ---------------------------------------------------------------------------

impl Module {
    fn render_rust(&self, source_file: &str, digest: &str) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "//! @generated by contracts-codegen v{GENERATOR_VERSION}. DO NOT EDIT."
        );
        let _ = writeln!(out, "//! source: specs/schemas/{source_file}");
        let _ = writeln!(
            out,
            "//! schema_digest: {digest} (canonical bytes, domain schema-bundle/0.1)"
        );
        let _ = writeln!(out, "//! policy: docs/adr/0006-code-generation-policy.md");
        let _ = writeln!(out, "//!");
        let _ = writeln!(
            out,
            "//! Shape-level bindings: conditionals, const refinements and cross-field"
        );
        let _ = writeln!(
            out,
            "//! constraints remain enforced by JSON Schema validation."
        );
        let _ = writeln!(out);
        let _ = writeln!(out, "#![allow(clippy::doc_markdown)]");
        let _ = writeln!(out);
        let _ = writeln!(out, "use serde::{{Deserialize, Serialize}};");
        for ty in &self.types {
            let _ = writeln!(out);
            out.push_str(&self.render_rust_type(ty));
        }
        out
    }

    fn rust_ty(&self, ty: &Ty, nullable: bool) -> String {
        let base = match ty {
            Ty::String => "String".to_string(),
            Ty::I64 => "i64".to_string(),
            Ty::Bool => "bool".to_string(),
            Ty::Any => "serde_json::Value".to_string(),
            Ty::AnyMap => "serde_json::Map<String, serde_json::Value>".to_string(),
            Ty::Map(value) => format!(
                "std::collections::BTreeMap<String, {}>",
                self.rust_ty(value, false)
            ),
            Ty::Vec(item) => format!("Vec<{}>", self.rust_ty(item, false)),
            Ty::Named { module, name } => {
                if *module == self.rust_mod_name {
                    name.clone()
                } else {
                    format!("crate::generated::{module}::{name}")
                }
            }
        };
        if nullable {
            format!("Option<{base}>")
        } else {
            base
        }
    }

    fn render_rust_type(&self, ty: &NamedType) -> String {
        let mut out = String::new();
        match ty {
            NamedType::Struct {
                name,
                doc,
                deny_unknown,
                fields,
            } => {
                if let Some(doc) = doc {
                    let _ = writeln!(out, "/// {doc}");
                }
                let _ = writeln!(
                    out,
                    "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]"
                );
                if *deny_unknown {
                    let _ = writeln!(out, "#[serde(deny_unknown_fields)]");
                }
                let _ = writeln!(out, "pub struct {name} {{");
                for field in fields {
                    if let Some(doc) = &field.doc {
                        let _ = writeln!(out, "    /// {doc}");
                    }
                    let rust_name = rust_field_ident(&field.json_name);
                    let inner = self.rust_ty(&field.ty, field.nullable);
                    match (field.required, field.nullable) {
                        (true, _) => {
                            let _ = writeln!(out, "    pub {rust_name}: {inner},");
                        }
                        (false, false) => {
                            let _ = writeln!(
                                out,
                                "    #[serde(default, skip_serializing_if = \"Option::is_none\")]"
                            );
                            let _ = writeln!(out, "    pub {rust_name}: Option<{inner}>,");
                        }
                        (false, true) => {
                            // Outer Option: member presence; inner Option: JSON null.
                            let _ = writeln!(
                                out,
                                "    #[serde(default, skip_serializing_if = \"Option::is_none\")]"
                            );
                            let _ = writeln!(out, "    pub {rust_name}: Option<{inner}>,");
                        }
                    }
                }
                let _ = writeln!(out, "}}");
            }
            NamedType::Enum { name, doc, values } => {
                if let Some(doc) = doc {
                    let _ = writeln!(out, "/// {doc}");
                }
                let _ = writeln!(
                    out,
                    "#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]"
                );
                let _ = writeln!(out, "pub enum {name} {{");
                for value in values {
                    let _ = writeln!(out, "    #[serde(rename = \"{value}\")]");
                    let _ = writeln!(out, "    {},", variant_name(value));
                }
                let _ = writeln!(out, "}}");
            }
            NamedType::Union {
                name,
                doc,
                variants,
            } => {
                if let Some(doc) = doc {
                    let _ = writeln!(out, "/// {doc}");
                }
                let _ = writeln!(
                    out,
                    "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]"
                );
                let _ = writeln!(out, "#[serde(untagged)]");
                let _ = writeln!(out, "pub enum {name} {{");
                for variant in variants {
                    let rust = self.rust_ty(variant, false);
                    let label = union_variant_label(variant);
                    let _ = writeln!(out, "    {label}({rust}),");
                }
                let _ = writeln!(out, "}}");
            }
            NamedType::Newtype { name, doc } => {
                if let Some(doc) = doc {
                    let _ = writeln!(out, "/// {doc}");
                }
                let _ = writeln!(
                    out,
                    "#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]"
                );
                let _ = writeln!(out, "#[serde(transparent)]");
                let _ = writeln!(out, "pub struct {name}(pub String);");
            }
            NamedType::Alias { name, doc, ty } => {
                if let Some(doc) = doc {
                    let _ = writeln!(out, "/// {doc}");
                }
                let _ = writeln!(out, "pub type {name} = {};", self.rust_ty(ty, false));
            }
        }
        out
    }

    // -----------------------------------------------------------------------
    // TypeScript rendering
    // -----------------------------------------------------------------------

    fn render_ts(&self, source_file: &str, digest: &str) -> String {
        let mut imports: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for ty in &self.types {
            self.collect_ts_imports(ty, &mut imports);
        }
        let mut out = String::new();
        let _ = writeln!(
            out,
            "// @generated by contracts-codegen v{GENERATOR_VERSION}. DO NOT EDIT."
        );
        let _ = writeln!(out, "// source: specs/schemas/{source_file}");
        let _ = writeln!(
            out,
            "// schema_digest: {digest} (canonical bytes, domain schema-bundle/0.1)"
        );
        let _ = writeln!(out, "// policy: docs/adr/0006-code-generation-policy.md");
        let _ = writeln!(out, "//");
        let _ = writeln!(
            out,
            "// Shape-level bindings: conditionals, const refinements and cross-field"
        );
        let _ = writeln!(
            out,
            "// constraints remain enforced by JSON Schema validation."
        );
        for (module_stem, names) in &imports {
            let list = names.iter().cloned().collect::<Vec<_>>().join(", ");
            let _ = writeln!(out);
            let _ = writeln!(out, "import type {{ {list} }} from \"./{module_stem}.js\";");
        }
        for ty in &self.types {
            let _ = writeln!(out);
            out.push_str(&self.render_ts_type(ty));
        }
        out
    }

    fn collect_ts_imports(&self, ty: &NamedType, out: &mut BTreeMap<String, BTreeSet<String>>) {
        let mut visit = |t: &Ty| self.collect_ty_imports(t, out);
        match ty {
            NamedType::Struct { fields, .. } => {
                for field in fields {
                    visit(&field.ty);
                }
            }
            NamedType::Union { variants, .. } => {
                for variant in variants {
                    visit(variant);
                }
            }
            NamedType::Alias { ty, .. } => visit(ty),
            NamedType::Enum { .. } | NamedType::Newtype { .. } => {}
        }
    }

    fn collect_ty_imports(&self, ty: &Ty, out: &mut BTreeMap<String, BTreeSet<String>>) {
        match ty {
            Ty::Map(value) => self.collect_ty_imports(value, out),
            Ty::Vec(item) => self.collect_ty_imports(item, out),
            Ty::Named { module, name } if *module != self.rust_mod_name => {
                let stem = module.replace('_', "-");
                out.entry(stem).or_default().insert(name.clone());
            }
            _ => {}
        }
    }

    fn ts_ty(&self, ty: &Ty, nullable: bool) -> String {
        let base = match ty {
            Ty::String => "string".to_string(),
            Ty::I64 => "number".to_string(),
            Ty::Bool => "boolean".to_string(),
            Ty::Any => "unknown".to_string(),
            Ty::AnyMap => "Record<string, unknown>".to_string(),
            Ty::Map(value) => format!("Record<string, {}>", self.ts_ty(value, false)),
            Ty::Vec(item) => format!("{}[]", self.ts_ty(item, false)),
            Ty::Named { name, .. } => name.clone(),
        };
        if nullable {
            format!("{base} | null")
        } else {
            base
        }
    }

    fn render_ts_type(&self, ty: &NamedType) -> String {
        let mut out = String::new();
        match ty {
            NamedType::Struct {
                name, doc, fields, ..
            } => {
                if let Some(doc) = doc {
                    let _ = writeln!(out, "/** {doc} */");
                }
                let _ = writeln!(out, "export interface {name} {{");
                for field in fields {
                    if let Some(doc) = &field.doc {
                        let _ = writeln!(out, "  /** {doc} */");
                    }
                    let ts = self.ts_ty(&field.ty, field.nullable);
                    if field.required {
                        let _ = writeln!(out, "  {}: {ts};", field.json_name);
                    } else {
                        let _ = writeln!(out, "  {}?: {ts};", field.json_name);
                    }
                }
                let _ = writeln!(out, "}}");
            }
            NamedType::Enum { name, doc, values } => {
                if let Some(doc) = doc {
                    let _ = writeln!(out, "/** {doc} */");
                }
                let list = values
                    .iter()
                    .map(|v| format!("\"{v}\""))
                    .collect::<Vec<_>>()
                    .join(" | ");
                let _ = writeln!(out, "export type {name} = {list};");
            }
            NamedType::Union {
                name,
                doc,
                variants,
            } => {
                if let Some(doc) = doc {
                    let _ = writeln!(out, "/** {doc} */");
                }
                let list = variants
                    .iter()
                    .map(|v| self.ts_ty(v, false))
                    .collect::<Vec<_>>()
                    .join(" | ");
                let _ = writeln!(out, "export type {name} = {list};");
            }
            NamedType::Newtype { name, doc } => {
                if let Some(doc) = doc {
                    let _ = writeln!(out, "/** {doc} */");
                }
                let _ = writeln!(out, "export type {name} = string;");
            }
            NamedType::Alias { name, doc, ty } => {
                if let Some(doc) = doc {
                    let _ = writeln!(out, "/** {doc} */");
                }
                let _ = writeln!(out, "export type {name} = {};", self.ts_ty(ty, false));
            }
        }
        out
    }
}

fn rust_field_ident(json_name: &str) -> String {
    const KEYWORDS: [&str; 5] = ["type", "ref", "use", "move", "const"];
    if KEYWORDS.contains(&json_name) {
        format!("r#{json_name}")
    } else {
        json_name.to_string()
    }
}

fn union_variant_label(ty: &Ty) -> String {
    match ty {
        Ty::String => "String".to_string(),
        Ty::I64 => "Integer".to_string(),
        Ty::Bool => "Boolean".to_string(),
        Ty::Named { name, .. } => name.clone(),
        Ty::Vec(item) => format!("{}List", union_variant_label(item)),
        Ty::Map(_) | Ty::AnyMap => "Object".to_string(),
        Ty::Any => "Value".to_string(),
    }
}

fn render_rust_mod_rs(modules: &[(String, String)]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "//! @generated by contracts-codegen v{GENERATOR_VERSION}. DO NOT EDIT."
    );
    let _ = writeln!(
        out,
        "//! Schema-generated bindings for the IMP-08 minimal core object set"
    );
    let _ = writeln!(
        out,
        "//! plus its $ref closure (ADR-0006; source specs/schemas/)."
    );
    let _ = writeln!(out);
    for (name, _) in modules {
        let _ = writeln!(out, "pub mod {name};");
    }
    out
}

fn render_ts_index(modules: &[(String, String)]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "// @generated by contracts-codegen v{GENERATOR_VERSION}. DO NOT EDIT."
    );
    let _ = writeln!(
        out,
        "// Schema-generated bindings for the IMP-08 minimal core object set"
    );
    let _ = writeln!(
        out,
        "// plus its $ref closure (ADR-0006; source specs/schemas/)."
    );
    let _ = writeln!(out, "//");
    let _ = writeln!(
        out,
        "// Namespaced re-exports: identical inline type names may exist in"
    );
    let _ = writeln!(
        out,
        "// several modules (each schema owns its inline definitions)."
    );
    let _ = writeln!(out);
    for (stem, _) in modules {
        let namespace = camel(stem);
        let _ = writeln!(out, "export * as {namespace} from \"./{stem}.js\";");
    }
    out
}

fn camel(stem: &str) -> String {
    let pascal_name = pascal(stem);
    let mut chars = pascal_name.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_lowercase(), chars.as_str()),
        None => pascal_name,
    }
}
