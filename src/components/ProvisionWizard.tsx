import { useEffect, useEffectEvent, useState, type FormEvent } from "react";
import type {
  MinecraftVersionOption,
  ProvisionValidationIssue,
  ServerProperties
} from "../types/api";
import { useAppStore } from "../store/appStore";

interface PropertyRow {
  id: string;
  key: string;
  value: string;
}

const wizardSteps = [
  { id: "version", label: "Version" },
  { id: "details", label: "Details" },
  { id: "properties", label: "Properties" },
  { id: "review", label: "Review" }
] as const;

const seededProperties = [
  ["motd", "Primary Survival"],
  ["server-port", "25565"],
  ["difficulty", "normal"],
  ["max-players", "20"],
  ["online-mode", "true"],
  ["pvp", "true"]
] satisfies Array<[string, string]>;

export function ProvisionWizard() {
  const {
    settings,
    runtimes,
    minecraftVersions,
    loadMinecraftVersions,
    validateProvisioning,
    provisionServer
  } = useAppStore();
  const [stepIndex, setStepIndex] = useState(0);
  const [name, setName] = useState("Primary Survival");
  const [minecraftVersion, setMinecraftVersion] = useState("1.21.4");
  const [memoryMb, setMemoryMb] = useState(4096);
  const [port, setPort] = useState(25565);
  const [eulaAccepted, setEulaAccepted] = useState(true);
  const [javaRuntimeId, setJavaRuntimeId] = useState("");
  const [targetRootDirectory, setTargetRootDirectory] = useState(
    settings?.defaultServerDirectory ?? ""
  );
  const [versionQuery, setVersionQuery] = useState("");
  const [propertyRows, setPropertyRows] = useState<PropertyRow[]>(() =>
    seededProperties.map(([key, value], index) => ({
      id: `${key}-${index}`,
      key,
      value
    }))
  );
  const [motdCustomized, setMotdCustomized] = useState(false);
  const [validationIssues, setValidationIssues] = useState<ProvisionValidationIssue[]>([]);
  const [normalizedTargetDirectory, setNormalizedTargetDirectory] = useState("");
  const [validating, setValidating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const runValidation = useEffectEvent((request: ReturnType<typeof buildValidationRequest>) => {
    setValidating(true);
    void validateProvisioning(request)
      .then((result) => {
        setValidationIssues(result.issues);
        setNormalizedTargetDirectory(result.normalizedTargetDirectory);
      })
      .catch((error: unknown) => {
        setValidationIssues([
          {
            field: "general",
            step: wizardSteps[stepIndex].id,
            message: error instanceof Error ? error.message : "Validation failed."
          }
        ]);
      })
      .finally(() => {
        setValidating(false);
      });
  });
  const ensureCatalogLoaded = useEffectEvent(() => {
    if (minecraftVersions.length === 0) {
      void loadMinecraftVersions();
    }
  });

  useEffect(() => {
    if (settings) {
      setTargetRootDirectory(settings.defaultServerDirectory);
    }
  }, [settings]);

  useEffect(() => {
    if (!javaRuntimeId && runtimes[0]) {
      setJavaRuntimeId(runtimes[0].id);
    }
  }, [javaRuntimeId, runtimes]);

  useEffect(() => {
    ensureCatalogLoaded();
  }, [ensureCatalogLoaded]);

  useEffect(() => {
    setPropertyRows((current) =>
      syncLinkedPropertyRows(current, {
        ...(motdCustomized ? {} : { motd: name }),
        "server-port": String(port)
      })
    );
  }, [motdCustomized, name, port]);

  useEffect(() => {
    const request = buildValidationRequest(
      name,
      minecraftVersion,
      targetRootDirectory,
      javaRuntimeId,
      memoryMb,
      port,
      propertyRows
    );
    const timeout = window.setTimeout(() => {
      runValidation(request);
    }, 250);

    return () => {
      window.clearTimeout(timeout);
    };
  }, [
    javaRuntimeId,
    memoryMb,
    minecraftVersion,
    name,
    port,
    propertyRows,
    runValidation,
    stepIndex,
    targetRootDirectory
  ]);

  if (!settings) {
    return null;
  }

  const selectedVersion = minecraftVersions.find((entry) => entry.id === minecraftVersion) ?? null;
  const selectedRuntime = runtimes.find((entry) => entry.id === javaRuntimeId) ?? null;
  const filteredVersions = minecraftVersions.filter((entry) =>
    entry.id.toLowerCase().includes(versionQuery.toLowerCase())
  );
  const targetDirectory = buildTargetDirectory(targetRootDirectory, name);
  const currentStep = wizardSteps[stepIndex].id;
  const currentStepIssues = validationIssues.filter((issue) => issue.step === currentStep);
  const reviewIssues = validationIssues;
  const canSubmit = reviewIssues.length === 0 && eulaAccepted && !submitting && !validating;

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitError(null);
    setSubmitting(true);
    try {
      await provisionServer({
        name,
        minecraftVersion,
        targetDirectory,
        javaRuntimeId,
        memoryMb,
        port,
        eulaAccepted,
        serverProperties: rowsToProperties(propertyRows)
      });
      setStepIndex(0);
    } catch (error) {
      setSubmitError(error instanceof Error ? error.message : "Provisioning failed.");
    } finally {
      setSubmitting(false);
    }
  }

  function updatePropertyRow(id: string, field: "key" | "value", value: string) {
    setPropertyRows((current) =>
      current.map((row) => {
        if (row.id !== id) {
          return row;
        }
        const nextRow = {
          ...row,
          [field]: value
        };
        if (field === "value" && row.key === "server-port") {
          const nextPort = Number(value);
          if (Number.isInteger(nextPort) && nextPort >= 1024 && nextPort <= 65535) {
            setPort(nextPort);
          }
        }
        if (field === "value" && row.key === "motd") {
          setMotdCustomized(value !== name);
        }
        return nextRow;
      })
    );
  }

  function addPropertyRow() {
    setPropertyRows((current) => [
      ...current,
      {
        id: `property-${Date.now()}`,
        key: "",
        value: ""
      }
    ]);
  }

  function removePropertyRow(id: string) {
    setPropertyRows((current) => current.filter((row) => row.id !== id));
  }

  return (
    <section className="panel provision-panel">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Provisioning</p>
          <h2>Guided Vanilla Setup</h2>
          <p className="muted">
            Select an official Mojang release, pin a Java runtime, validate the install target,
            and define server properties before first boot.
          </p>
        </div>
      </div>
      <div className="wizard-steps" role="tablist" aria-label="Provisioning steps">
        {wizardSteps.map((step, index) => (
          <button
            key={step.id}
            className={index === stepIndex ? "wizard-step active" : "wizard-step"}
            onClick={() => setStepIndex(index)}
            type="button"
          >
            <span>{index + 1}</span>
            {step.label}
          </button>
        ))}
      </div>
      <form className="form-grid" onSubmit={handleSubmit}>
        {currentStep === "version" && (
          <div className="wizard-grid">
            <label>
              Search versions
              <input
                placeholder="Search Mojang releases"
                value={versionQuery}
                onChange={(event) => setVersionQuery(event.target.value)}
              />
            </label>
            <div className="version-catalog" role="list">
              {filteredVersions.map((version) => (
                <button
                  key={version.id}
                  className={
                    version.id === minecraftVersion
                      ? "version-option active"
                      : "version-option"
                  }
                  onClick={() => setMinecraftVersion(version.id)}
                  type="button"
                >
                  <strong>{version.id}</strong>
                  <span>
                    {new Date(version.publishedAt).toLocaleDateString()} · Java{" "}
                    {version.requiredJavaMajor ?? "Unknown"}
                  </span>
                </button>
              ))}
            </div>
            <label>
              Java runtime
              <select
                value={javaRuntimeId}
                onChange={(event) => setJavaRuntimeId(event.target.value)}
              >
                <option value="">Select a runtime</option>
                {runtimes.map((runtime) => (
                  <option key={runtime.id} value={runtime.id}>
                    {runtime.vendor} {runtime.version} ({runtime.architecture})
                  </option>
                ))}
              </select>
            </label>
            <div className="list-card">
              <strong>Compatibility</strong>
              <p className="muted">
                {formatCompatibility(selectedVersion, selectedRuntime)}
              </p>
            </div>
          </div>
        )}

        {currentStep === "details" && (
          <div className="wizard-grid">
            <label>
              Server name
              <input value={name} onChange={(event) => setName(event.target.value)} />
            </label>
            <label>
              Install root
              <input
                value={targetRootDirectory}
                onChange={(event) => setTargetRootDirectory(event.target.value)}
              />
            </label>
            <label>
              Provisioned directory
              <input disabled value={normalizedTargetDirectory || targetDirectory} />
            </label>
            <label>
              Memory (MB)
              <input
                min={1024}
                step={512}
                type="number"
                value={memoryMb}
                onChange={(event) => setMemoryMb(Number(event.target.value))}
              />
            </label>
            <label>
              Port
              <input
                min={1024}
                max={65535}
                type="number"
                value={port}
                onChange={(event) => setPort(Number(event.target.value))}
              />
            </label>
          </div>
        )}

        {currentStep === "properties" && (
          <div className="stack">
            <div className="section-heading">
              <div>
                <h3>server.properties</h3>
                <p className="muted">
                  Seeded defaults stay linked to the main inputs. Add any extra Vanilla property
                  keys you need before first start.
                </p>
              </div>
              <button onClick={addPropertyRow} type="button">
                Add property
              </button>
            </div>
            <div className="property-grid">
              {propertyRows.map((row) => (
                <div key={row.id} className="property-row">
                  <input
                    aria-label={`Property key ${row.id}`}
                    placeholder="property-key"
                    value={row.key}
                    onChange={(event) => updatePropertyRow(row.id, "key", event.target.value)}
                  />
                  <input
                    aria-label={`Property value ${row.id}`}
                    placeholder="value"
                    value={row.value}
                    onChange={(event) => updatePropertyRow(row.id, "value", event.target.value)}
                  />
                  <button onClick={() => removePropertyRow(row.id)} type="button">
                    Remove
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {currentStep === "review" && (
          <div className="stack">
            <div className="review-grid">
              <article className="list-card">
                <strong>Release</strong>
                <span>{minecraftVersion}</span>
                <span className="muted">
                  Java requirement: {selectedVersion?.requiredJavaMajor ?? "Unknown"}
                </span>
              </article>
              <article className="list-card">
                <strong>Runtime</strong>
                <span>
                  {selectedRuntime
                    ? `${selectedRuntime.vendor} ${selectedRuntime.version}`
                    : "No runtime selected"}
                </span>
              </article>
              <article className="list-card">
                <strong>Install target</strong>
                <span>{normalizedTargetDirectory || targetDirectory}</span>
              </article>
              <article className="list-card">
                <strong>Server sizing</strong>
                <span>
                  {memoryMb} MB · Port {port}
                </span>
              </article>
            </div>
            <div className="list-card">
              <strong>Effective properties</strong>
              <pre className="property-preview">
                {Object.entries(rowsToProperties(propertyRows))
                  .map(([key, value]) => `${key}=${value}`)
                  .join("\n")}
              </pre>
            </div>
            <label className="checkbox">
              <input
                checked={eulaAccepted}
                type="checkbox"
                onChange={(event) => setEulaAccepted(event.target.checked)}
              />
              I accept the Minecraft server EULA for this provisioned server.
            </label>
          </div>
        )}

        {currentStepIssues.length > 0 && (
          <div className="validation-list" role="alert">
            {currentStepIssues.map((issue) => (
              <p key={`${issue.step}-${issue.field}-${issue.message}`}>{issue.message}</p>
            ))}
          </div>
        )}
        {currentStep === "review" && reviewIssues.length > 0 && (
          <div className="validation-list" role="alert">
            {reviewIssues.map((issue) => (
              <p key={`${issue.step}-${issue.field}-${issue.message}`}>{issue.message}</p>
            ))}
          </div>
        )}
        {submitError && (
          <div className="validation-list" role="alert">
            <p>{submitError}</p>
          </div>
        )}

        <div className="wizard-actions">
          <button
            disabled={stepIndex === 0}
            onClick={() => setStepIndex((current) => Math.max(0, current - 1))}
            type="button"
          >
            Back
          </button>
          {stepIndex < wizardSteps.length - 1 ? (
            <button
              onClick={() =>
                setStepIndex((current) => Math.min(wizardSteps.length - 1, current + 1))
              }
              type="button"
            >
              Next
            </button>
          ) : (
            <button disabled={!canSubmit} type="submit">
              {submitting ? "Provisioning..." : validating ? "Validating..." : "Provision Server"}
            </button>
          )}
        </div>
      </form>
    </section>
  );
}

function buildTargetDirectory(root: string, name: string) {
  const normalizedRoot = root.endsWith("/") ? root.slice(0, -1) : root;
  return `${normalizedRoot}/${slugify(name)}`;
}

function slugify(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function syncLinkedPropertyRows(
  rows: PropertyRow[],
  linkedValues: Record<string, string>
): PropertyRow[] {
  const nextRows = [...rows];
  Object.entries(linkedValues).forEach(([key, value]) => {
    const index = nextRows.findIndex((row) => row.key === key);
    if (index === -1) {
      nextRows.push({
        id: `${key}-${Date.now()}`,
        key,
        value
      });
      return;
    }
    nextRows[index] = {
      ...nextRows[index],
      value
    };
  });
  return nextRows;
}

function rowsToProperties(rows: PropertyRow[]): ServerProperties {
  const properties: ServerProperties = {};
  rows.forEach((row) => {
    if (row.key.trim()) {
      properties[row.key.trim()] = row.value;
    }
  });
  return properties;
}

function buildValidationRequest(
  name: string,
  minecraftVersion: string,
  targetRootDirectory: string,
  javaRuntimeId: string,
  memoryMb: number,
  port: number,
  propertyRows: PropertyRow[]
) {
  return {
    name,
    minecraftVersion,
    targetDirectory: buildTargetDirectory(targetRootDirectory, name),
    javaRuntimeId: javaRuntimeId || undefined,
    memoryMb,
    port,
    serverProperties: rowsToProperties(propertyRows)
  };
}

function formatCompatibility(
  version: MinecraftVersionOption | null,
  runtime: { vendor: string; version: string } | null
) {
  if (!runtime) {
    return "Select a Java runtime to validate compatibility.";
  }
  if (!version?.requiredJavaMajor) {
    return `Using ${runtime.vendor} ${runtime.version}. Mojang does not expose a Java requirement for this release.`;
  }
  const runtimeMajor = extractJavaMajor(runtime.version);
  if (runtimeMajor !== null && runtimeMajor >= version.requiredJavaMajor) {
    return `${runtime.vendor} ${runtime.version} satisfies the Java ${version.requiredJavaMajor}+ requirement for ${version.id}.`;
  }
  return `${runtime.vendor} ${runtime.version} may be too old for ${version.id}. Java ${version.requiredJavaMajor}+ is required.`;
}

function extractJavaMajor(version: string) {
  if (version.startsWith("1.")) {
    return Number.parseInt(version.split(".")[1] ?? "", 10) || null;
  }
  return Number.parseInt(version.split(".")[0] ?? "", 10) || null;
}
