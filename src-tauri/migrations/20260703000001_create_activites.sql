CREATE TABLE IF NOT EXISTS activites (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    nom             TEXT NOT NULL,
    description     TEXT,
    capacite_max    INTEGER
);

CREATE INDEX IF NOT EXISTS idx_activites_nom ON activites(nom);
