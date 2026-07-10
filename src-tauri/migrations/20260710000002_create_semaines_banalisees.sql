CREATE TABLE IF NOT EXISTS semaines_banalisees (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    activite_id     INTEGER NOT NULL,
    date_debut      TEXT NOT NULL,
    motif           TEXT,
    annee_scolaire  TEXT NOT NULL,
    FOREIGN KEY (activite_id) REFERENCES activites(id)
);

CREATE INDEX IF NOT EXISTS idx_semaines_banalisees_activite ON semaines_banalisees(activite_id);
