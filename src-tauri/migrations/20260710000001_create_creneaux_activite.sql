CREATE TABLE IF NOT EXISTS creneaux_activite (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    activite_id     INTEGER NOT NULL,
    jour_semaine    INTEGER NOT NULL CHECK (jour_semaine BETWEEN 1 AND 7),
    heure_debut     TEXT NOT NULL,
    heure_fin       TEXT NOT NULL,
    annee_scolaire  TEXT NOT NULL,
    FOREIGN KEY (activite_id) REFERENCES activites(id),
    CHECK (heure_debut < heure_fin)
);

CREATE INDEX IF NOT EXISTS idx_creneaux_activite_activite ON creneaux_activite(activite_id);
CREATE INDEX IF NOT EXISTS idx_creneaux_activite_annee ON creneaux_activite(annee_scolaire);
