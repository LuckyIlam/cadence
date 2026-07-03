CREATE TABLE IF NOT EXISTS tarifs_activite (
    activite_id     INTEGER NOT NULL,
    annee_scolaire  TEXT NOT NULL,
    tarif           REAL NOT NULL,
    FOREIGN KEY (activite_id) REFERENCES activites(id),
    UNIQUE(activite_id, annee_scolaire)
);

CREATE INDEX IF NOT EXISTS idx_tarifs_activite ON tarifs_activite(activite_id);
