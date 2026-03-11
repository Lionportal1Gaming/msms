import { useState, type FormEvent } from "react";
import { useAppStore } from "../store/appStore";

export function AuthGate() {
  const { passwordConfigured, setPassword, unlock } = useAppStore();
  const [password, setPasswordValue] = useState("");

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (passwordConfigured) {
      await unlock({ password });
    } else {
      await setPassword({ password });
    }
    setPasswordValue("");
  }

  return (
    <main className="auth-shell">
      <section className="panel auth-panel">
        <p className="eyebrow">MSMS Access</p>
        <h1>{passwordConfigured ? "Unlock Workspace" : "Secure The App"}</h1>
        <p className="muted">
          {passwordConfigured
            ? "Enter the local application password to unlock operations."
            : "Create the initial local password for this desktop installation."}
        </p>
        <form className="auth-form" onSubmit={handleSubmit}>
          <label>
            Password
            <input
              required
              minLength={12}
              type="password"
              value={password}
              onChange={(event) => setPasswordValue(event.target.value)}
            />
          </label>
          <button type="submit">
            {passwordConfigured ? "Unlock" : "Create password"}
          </button>
        </form>
      </section>
    </main>
  );
}
