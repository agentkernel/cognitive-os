import { DigestChip } from "../../components/DigestChip";
import { type IdentityCardView } from "../../data/projections/agents";

/**
 * Identity documents — docs/design/16 §3. The nine-key merge from
 * identities.ts, each card source-labeled. Unknown stays unknown.
 */
export function IdentityCards({ cards }: { cards: IdentityCardView[] }) {
  return (
    <ul className="cp-identity-grid">
      {cards.map((card) => (
        <li key={card.key} className="cp-identity-card">
          <h4 className="cp-identity-key">{card.key}</h4>
          <p className="cp-mono">
            {card.value === "unknown" ? (
              "unknown"
            ) : (
              <DigestChip value={card.value} label={card.key} />
            )}
          </p>
          <p className="cp-quiet">{card.source}</p>
          {card.caption ? <p className="cp-reason">{card.caption}</p> : null}
        </li>
      ))}
    </ul>
  );
}
