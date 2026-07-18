import { getNumeroSemaineISO, jourSemaineTexte, type PlanningCreneau } from "../types";

interface PlanningHebdoProps {
  creneaux: PlanningCreneau[];
  dateLundi: Date;
  onSemainePrecedente: () => void;
  onSemaineSuivante: () => void;
}

const HEURES = Array.from({ length: 13 }, (_, i) => i + 8);
const HAUTEUR_LIGNE = 60;

function parseHeure(heure: string): { h: number; m: number } {
  const parts = heure.split(":");
  return { h: Number(parts[0]), m: Number(parts[1]) };
}

function posY(heure: string): number {
  const { h, m } = parseHeure(heure);
  return (h + m / 60 - 8) * HAUTEUR_LIGNE;
}

function hauteurBloc(debut: string, fin: string): number {
  const { h: h1, m: m1 } = parseHeure(debut);
  const { h: h2, m: m2 } = parseHeure(fin);
  return (h2 + m2 / 60 - (h1 + m1 / 60)) * HAUTEUR_LIGNE;
}

export default function PlanningHebdo({
  creneaux,
  dateLundi,
  onSemainePrecedente,
  onSemaineSuivante,
}: PlanningHebdoProps) {
  const semaineNum = getNumeroSemaineISO(dateLundi);
  const dateLundiStr = dateLundi.toLocaleDateString("fr-FR", {
    day: "numeric",
    month: "short",
    year: "numeric",
  });

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <button
          type="button"
          onClick={onSemainePrecedente}
          className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-50 transition-colors"
        >
          &larr; Précédente
        </button>
        <span className="text-sm font-medium text-gray-700">
          Semaine {semaineNum} (lun {dateLundiStr})
        </span>
        <button
          type="button"
          onClick={onSemaineSuivante}
          className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-50 transition-colors"
        >
          Suivante &rarr;
        </button>
      </div>

      {creneaux.length === 0 ? (
        <p className="text-gray-500 text-center py-8 bg-white rounded-lg border border-gray-200">
          Aucune activité cette semaine
        </p>
      ) : (
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          <div
            className="grid"
            style={{
              gridTemplateColumns: "60px repeat(7, 1fr)",
            }}
          >
            <div />
            {[1, 2, 3, 4, 5, 6, 7].map((j) => (
              <div key={j} className="text-xs font-medium text-gray-500 text-center py-2 border-b border-gray-200">
                {jourSemaineTexte(j)}
              </div>
            ))}
          </div>
          <div className="relative" style={{ height: `${HEURES.length * HAUTEUR_LIGNE}px` }}>
            <div
              className="absolute inset-0 grid"
              style={{
                gridTemplateColumns: "60px repeat(7, 1fr)",
                gridTemplateRows: `repeat(${HEURES.length}, ${HAUTEUR_LIGNE}px)`,
              }}
            >
              {HEURES.map((h) => (
                <div
                  key={h}
                  className="text-xs text-gray-400 text-right pr-2 border-r border-b border-gray-100"
                  style={{ lineHeight: `${HAUTEUR_LIGNE}px` }}
                >
                  {h}h00
                </div>
              ))}
              {HEURES.map((h) =>
                [1, 2, 3, 4, 5, 6, 7].map((j) => (
                  <div key={`${h}-${j}`} className="border-r border-b border-gray-100" />
                )),
              )}
            </div>
            {creneaux.map((pc) => {
              const y = posY(pc.creneau.heure_debut);
              const h = hauteurBloc(pc.creneau.heure_debut, pc.creneau.heure_fin);
              return (
                <div
                  key={pc.creneau.id}
                  className="absolute rounded px-1.5 py-0.5 text-xs overflow-hidden"
                  style={{
                    left: `calc(60px + ${(pc.creneau.jour_semaine - 1) * (100 / 7)}% + 2px)`,
                    width: `calc(${100 / 7}% - 4px)`,
                    top: `${y}px`,
                    height: `${h}px`,
                    minHeight: "20px",
                    backgroundColor: pc.role === "encadrant" ? "#dbeafe" : "#dcfce7",
                    border: `1px solid ${pc.role === "encadrant" ? "#93c5fd" : "#86efac"}`,
                  }}
                >
                  <div className="font-medium truncate">{pc.activite.nom}</div>
                  <div className="truncate opacity-75">{pc.role === "encadrant" ? "Encadrant" : "Participant"}</div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
