CREATE TABLE IF NOT EXISTS parametres (
    id               INTEGER PRIMARY KEY CHECK (id = 1),
    heure_ouverture  TEXT NOT NULL,
    heure_fermeture  TEXT NOT NULL,
    CHECK (heure_ouverture < heure_fermeture)
);

-- Plage par défaut : ouverture 08:00 – fermeture 20:00
INSERT INTO parametres (id, heure_ouverture, heure_fermeture)
VALUES (1, '08:00', '20:00');
