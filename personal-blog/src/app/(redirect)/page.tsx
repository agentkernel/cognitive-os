import { permanentRedirect } from "next/navigation";

export default function RootRedirectPage(): never {
  permanentRedirect("/zh");
}
