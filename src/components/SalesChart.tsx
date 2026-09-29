import { useState } from "react";
import type { DailyProductSales } from "../types";

interface Props {
  data: DailyProductSales[];
  currency: string;
}

const W = 800, H = 260, PL = 40, PR = 16, PT = 16, PB = 32;

const fmt = (d: string) => `${d.slice(8, 10)}/${d.slice(5, 7)}`;

export default function SalesChart({ data, currency }: Props) {
  const [hover, setHover] = useState<number | null>(null);

  if (data.length === 0) {
    return <p className="py-10 text-center text-sm text-gray-400">Chargement…</p>;
  }

  const total = data.reduce((s, d) => s + d.total_quantity, 0);
  if (total === 0) {
    return <p className="py-10 text-center text-sm text-gray-400">Aucune vente sur les 30 derniers jours</p>;
  }

  const yMax = Math.max(4, ...data.map((d) => d.total_quantity));
  const n = data.length;
  const x = (i: number) => PL + (i * (W - PL - PR)) / (n - 1);
  const y = (q: number) => PT + (1 - q / yMax) * (H - PT - PB);

  const line = data.map((d, i) => `${i === 0 ? "M" : "L"}${x(i)},${y(d.total_quantity)}`).join(" ");
  const area = `${line} L${x(n - 1)},${y(0)} L${x(0)},${y(0)} Z`;
  const ticks = [0, 0.25, 0.5, 0.75, 1].map((t) => Math.round(t * yMax));
  const h = hover !== null ? data[hover] : null;

  return (
    <div className="relative w-full">
      <svg viewBox={`0 0 ${W} ${H}`} className="w-full">
        {/* grille + axe Y */}
        {ticks.map((t) => (
          <g key={t}>
            <line x1={PL} x2={W - PR} y1={y(t)} y2={y(t)} stroke="#e5e7eb" strokeDasharray="4 4" />
            <text x={PL - 8} y={y(t) + 4} textAnchor="end" fontSize="11" fill="#9ca3af">{t}</text>
          </g>
        ))}

        {/* axe X : une étiquette tous les 5 jours */}
        {data.map((d, i) =>
          i % 5 === 0 || i === n - 1 ? (
            <text key={d.date} x={x(i)} y={H - 10} textAnchor="middle" fontSize="11" fill="#9ca3af">
              {fmt(d.date)}
            </text>
          ) : null
        )}

        {/* aire + courbe */}
        <path d={area} fill="#15803d" opacity="0.1" />
        <path d={line} fill="none" stroke="#15803d" strokeWidth="2.5" strokeLinejoin="round" strokeLinecap="round" />

        {/* point survolé */}
        {h && hover !== null && (
          <>
            <line x1={x(hover)} x2={x(hover)} y1={PT} y2={H - PB} stroke="#15803d" opacity="0.3" />
            <circle cx={x(hover)} cy={y(h.total_quantity)} r="5" fill="#15803d" stroke="white" strokeWidth="2" />
          </>
        )}

        {/* zones de survol */}
        {data.map((_, i) => (
          <rect
            key={i}
            x={x(i) - (W - PL - PR) / (n - 1) / 2}
            y={PT}
            width={(W - PL - PR) / (n - 1)}
            height={H - PT - PB}
            fill="transparent"
            onMouseEnter={() => setHover(i)}
            onMouseLeave={() => setHover(null)}
          />
        ))}
      </svg>

      {h && hover !== null && (
        <div
          className="pointer-events-none absolute rounded-lg bg-gray-800 px-3 py-2 text-xs text-white shadow"
          style={{
            left: `${(x(hover) / W) * 100}%`,
            top: 0,
            transform: `translateX(${hover > n / 2 ? "-105%" : "5%"})`,
          }}
        >
          <div className="font-medium">{fmt(h.date)}</div>
          <div>{h.total_quantity} unité(s)</div>
          <div>{h.total_revenue.toLocaleString()} {currency}</div>
        </div>
      )}
    </div>
  );
}