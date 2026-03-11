interface StatCardProps {
  label: string;
  value: string;
  detail: string;
}

export function StatCard({ label, value, detail }: StatCardProps) {
  return (
    <article className="panel stat-card">
      <p className="muted">{label}</p>
      <strong>{value}</strong>
      <span>{detail}</span>
    </article>
  );
}

