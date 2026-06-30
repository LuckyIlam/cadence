CREATE TABLE IF NOT EXISTS adhesions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    personne_id     INTEGER NOT NULL,
    annee_scolaire  TEXT NOT NULL,
    reglee          INTEGER NOT NULL DEFAULT 0,
    note_paiement   TEXT,
    FOREIGN KEY (personne_id) REFERENCES personnes_physiques(id),
    UNIQUE(personne_id, annee_scolaire)
);

CREATE INDEX IF NOT EXISTS idx_adhesions_personne ON adhesions(personne_id);
