CREATE TABLE IF NOT EXISTS personnes_physiques (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    nom             TEXT NOT NULL,
    prenom          TEXT NOT NULL,
    date_naissance  TEXT NOT NULL,
    email           TEXT,
    telephone       TEXT,
    responsable_id  INTEGER,
    FOREIGN KEY (responsable_id) REFERENCES personnes_physiques(id)
);

CREATE INDEX IF NOT EXISTS idx_personnes_nom ON personnes_physiques(nom);
CREATE INDEX IF NOT EXISTS idx_personnes_prenom ON personnes_physiques(prenom);

-- Vérification que les mineurs ont un responsable (validation âge approximative)
-- La validation précise (responsable majeur) est faite côté Rust
CREATE TRIGGER IF NOT EXISTS trg_personnes_check_responsable
BEFORE INSERT ON personnes_physiques
FOR EACH ROW
WHEN (
    CAST(strftime('%Y', 'now') AS INTEGER) - CAST(strftime('%Y', NEW.date_naissance) AS INTEGER) -
    CASE WHEN strftime('%m-%d', 'now') < strftime('%m-%d', NEW.date_naissance) THEN 1 ELSE 0 END
) < 18 AND NEW.responsable_id IS NULL
BEGIN
    SELECT RAISE(ABORT, 'Un mineur doit avoir un responsable légal');
END;
