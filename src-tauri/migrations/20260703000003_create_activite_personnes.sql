CREATE TABLE IF NOT EXISTS activite_personnes (
    activite_id     INTEGER NOT NULL,
    personne_id     INTEGER NOT NULL,
    annee_scolaire  TEXT NOT NULL,
    role            TEXT NOT NULL CHECK (role IN ('encadrant', 'participant')),
    FOREIGN KEY (activite_id) REFERENCES activites(id),
    FOREIGN KEY (personne_id) REFERENCES personnes_physiques(id),
    UNIQUE(activite_id, personne_id, annee_scolaire)
);

CREATE INDEX IF NOT EXISTS idx_activite_personnes_activite ON activite_personnes(activite_id);
CREATE INDEX IF NOT EXISTS idx_activite_personnes_personne ON activite_personnes(personne_id);
