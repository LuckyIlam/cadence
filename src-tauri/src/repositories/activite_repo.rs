use sqlx::SqlitePool;

use crate::domain::activite::{
    Activite, CreateActivite, CreateLiaisonActivitePersonne, CreateTarifActivite,
    LiaisonActivitePersonne, PersonneActivite, TarifActivite, UpdateActivite,
};

pub async fn create(pool: &SqlitePool, input: CreateActivite) -> Result<Activite, sqlx::Error> {
    let row = sqlx::query_as::<_, Activite>(
        "INSERT INTO activites (nom, description, capacite_max)
         VALUES (?, ?, ?)
         RETURNING *",
    )
    .bind(&input.nom)
    .bind(&input.description)
    .bind(input.capacite_max)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn update(
    pool: &SqlitePool,
    id: i64,
    input: UpdateActivite,
) -> Result<Activite, sqlx::Error> {
    let row = sqlx::query_as::<_, Activite>(
        "UPDATE activites
         SET nom = ?, description = ?, capacite_max = ?
         WHERE id = ?
         RETURNING *",
    )
    .bind(&input.nom)
    .bind(&input.description)
    .bind(input.capacite_max)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Activite>, sqlx::Error> {
    let row = sqlx::query_as::<_, Activite>("SELECT * FROM activites WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

pub async fn upsert_tarif(
    pool: &SqlitePool,
    input: CreateTarifActivite,
) -> Result<TarifActivite, sqlx::Error> {
    let row = sqlx::query_as::<_, TarifActivite>(
        "INSERT INTO tarifs_activite (activite_id, annee_scolaire, tarif)
         VALUES (?, ?, ?)
         ON CONFLICT(activite_id, annee_scolaire)
         DO UPDATE SET tarif = excluded.tarif
         RETURNING *",
    )
    .bind(input.activite_id)
    .bind(&input.annee_scolaire)
    .bind(input.tarif)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn get_tarif(
    pool: &SqlitePool,
    activite_id: i64,
    annee_scolaire: &str,
) -> Result<Option<TarifActivite>, sqlx::Error> {
    let row = sqlx::query_as::<_, TarifActivite>(
        "SELECT * FROM tarifs_activite WHERE activite_id = ? AND annee_scolaire = ?",
    )
    .bind(activite_id)
    .bind(annee_scolaire)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn ajouter_personne(
    pool: &SqlitePool,
    input: CreateLiaisonActivitePersonne,
) -> Result<LiaisonActivitePersonne, sqlx::Error> {
    let row = sqlx::query_as::<_, LiaisonActivitePersonne>(
        "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
         VALUES (?, ?, ?, ?)
         RETURNING *",
    )
    .bind(input.activite_id)
    .bind(input.personne_id)
    .bind(&input.annee_scolaire)
    .bind(&input.role)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn retirer_personne(
    pool: &SqlitePool,
    activite_id: i64,
    personne_id: i64,
    annee_scolaire: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM activite_personnes WHERE activite_id = ? AND personne_id = ? AND annee_scolaire = ?",
    )
    .bind(activite_id)
    .bind(personne_id)
    .bind(annee_scolaire)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn compter_participants(
    pool: &SqlitePool,
    activite_id: i64,
    annee_scolaire: &str,
) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM activite_personnes
         WHERE activite_id = ? AND annee_scolaire = ? AND role = 'participant'",
    )
    .bind(activite_id)
    .bind(annee_scolaire)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

pub async fn trouver_liaison(
    pool: &SqlitePool,
    activite_id: i64,
    personne_id: i64,
    annee_scolaire: &str,
) -> Result<Option<LiaisonActivitePersonne>, sqlx::Error> {
    let row = sqlx::query_as::<_, LiaisonActivitePersonne>(
        "SELECT * FROM activite_personnes
         WHERE activite_id = ? AND personne_id = ? AND annee_scolaire = ?",
    )
    .bind(activite_id)
    .bind(personne_id)
    .bind(annee_scolaire)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn lister_encadrants(
    pool: &SqlitePool,
    activite_id: i64,
    annee_scolaire: &str,
) -> Result<Vec<PersonneActivite>, sqlx::Error> {
    let rows = sqlx::query_as::<_, PersonneActivite>(
        "SELECT pp.id, pp.nom, pp.prenom
         FROM activite_personnes ap
         JOIN personnes_physiques pp ON pp.id = ap.personne_id
         WHERE ap.activite_id = ? AND ap.annee_scolaire = ? AND ap.role = 'encadrant'
         ORDER BY pp.nom, pp.prenom",
    )
    .bind(activite_id)
    .bind(annee_scolaire)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn lister_participants(
    pool: &SqlitePool,
    activite_id: i64,
    annee_scolaire: &str,
) -> Result<Vec<PersonneActivite>, sqlx::Error> {
    let rows = sqlx::query_as::<_, PersonneActivite>(
        "SELECT pp.id, pp.nom, pp.prenom
         FROM activite_personnes ap
         JOIN personnes_physiques pp ON pp.id = ap.personne_id
         WHERE ap.activite_id = ? AND ap.annee_scolaire = ? AND ap.role = 'participant'
         ORDER BY pp.nom, pp.prenom",
    )
    .bind(activite_id)
    .bind(annee_scolaire)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct ActivitePersonneRow {
    id: i64,
    nom: String,
    description: Option<String>,
    capacite_max: Option<i64>,
    role: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct ActiviteAnneeRow {
    id: i64,
    nom: String,
    description: Option<String>,
    capacite_max: Option<i64>,
    tarif: Option<f64>,
    nb_participants: i64,
}

pub async fn lister_activites_personne(
    pool: &SqlitePool,
    personne_id: i64,
) -> Result<Vec<crate::domain::activite::ActivitePersonne>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ActivitePersonneRow>(
        "SELECT a.id, a.nom, a.description, a.capacite_max, ap.role
         FROM activite_personnes ap
         JOIN activites a ON a.id = ap.activite_id
         WHERE ap.personne_id = ?
         ORDER BY a.nom",
    )
    .bind(personne_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| crate::domain::activite::ActivitePersonne {
            activite: Activite {
                id: r.id,
                nom: r.nom,
                description: r.description,
                capacite_max: r.capacite_max,
            },
            role: r.role,
        })
        .collect())
}

pub async fn lister_annees_disponibles(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT annee_scolaire FROM tarifs_activite ORDER BY annee_scolaire DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn lister_activites_par_annee(
    pool: &SqlitePool,
    annee_scolaire: &str,
) -> Result<Vec<(Activite, Option<f64>, i64)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ActiviteAnneeRow>(
        "SELECT a.id, a.nom, a.description, a.capacite_max, ta.tarif,
                (SELECT COUNT(*) FROM activite_personnes ap2
                 WHERE ap2.activite_id = a.id AND ap2.annee_scolaire = ? AND ap2.role = 'participant') AS nb_participants
         FROM activites a
         JOIN tarifs_activite ta ON ta.activite_id = a.id AND ta.annee_scolaire = ?
         ORDER BY a.nom",
    )
    .bind(annee_scolaire)
    .bind(annee_scolaire)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            (
                Activite {
                    id: r.id,
                    nom: r.nom,
                    description: r.description,
                    capacite_max: r.capacite_max,
                },
                r.tarif,
                r.nb_participants,
            )
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::activite::CreateActivite;
    use sqlx::SqlitePool;

    async fn setup_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("failed to create test pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");
        pool
    }

    async fn seed_activite(pool: &SqlitePool, nom: &str) -> Activite {
        create(
            pool,
            CreateActivite {
                nom: nom.to_string(),
                description: None,
                capacite_max: None,
                annee_scolaire: None,
                tarif: None,
            },
        )
        .await
        .expect("failed to seed activite")
    }

    async fn seed_personne(pool: &SqlitePool) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO personnes_physiques (nom, prenom, date_naissance)
             VALUES (?, ?, ?) RETURNING id",
        )
        .bind("Test")
        .bind("User")
        .bind("2000-01-15")
        .fetch_one(pool)
        .await
        .expect("failed to seed personne")
    }

    #[tokio::test]
    async fn test_create_activite() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;
        assert_eq!(a.nom, "Poterie");
        assert_eq!(a.id, 1);
    }

    #[tokio::test]
    async fn test_liste_activites_par_annee() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;

        upsert_tarif(
            &pool,
            CreateTarifActivite {
                activite_id: a.id,
                annee_scolaire: "2025-2026".into(),
                tarif: 200.0,
            },
        )
        .await
        .unwrap();

        let list = lister_activites_par_annee(&pool, "2025-2026")
            .await
            .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0.nom, "Poterie");
    }

    #[tokio::test]
    async fn test_tarif_upsert() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;

        let t = upsert_tarif(
            &pool,
            CreateTarifActivite {
                activite_id: a.id,
                annee_scolaire: "2025-2026".into(),
                tarif: 200.0,
            },
        )
        .await
        .unwrap();
        assert_eq!(t.tarif, 200.0);

        let t2 = upsert_tarif(
            &pool,
            CreateTarifActivite {
                activite_id: a.id,
                annee_scolaire: "2025-2026".into(),
                tarif: 220.0,
            },
        )
        .await
        .unwrap();
        assert_eq!(t2.tarif, 220.0);
    }

    #[tokio::test]
    async fn test_ajouter_personne() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;

        let liaison = ajouter_personne(
            &pool,
            CreateLiaisonActivitePersonne {
                activite_id: a.id,
                personne_id: pid,
                annee_scolaire: "2025-2026".into(),
                role: "participant".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(liaison.role, "participant");

        let participants = lister_participants(&pool, a.id, "2025-2026").await.unwrap();
        assert_eq!(participants.len(), 1);
    }

    #[tokio::test]
    async fn test_lister_activites_personne() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;

        ajouter_personne(
            &pool,
            CreateLiaisonActivitePersonne {
                activite_id: a.id,
                personne_id: pid,
                annee_scolaire: "2025-2026".into(),
                role: "participant".into(),
            },
        )
        .await
        .unwrap();

        let list = lister_activites_personne(&pool, pid).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].activite.id, a.id);
        assert_eq!(list[0].role, "participant");
    }

    #[tokio::test]
    async fn test_retirer_personne() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;

        ajouter_personne(
            &pool,
            CreateLiaisonActivitePersonne {
                activite_id: a.id,
                personne_id: pid,
                annee_scolaire: "2025-2026".into(),
                role: "participant".into(),
            },
        )
        .await
        .unwrap();

        retirer_personne(&pool, a.id, pid, "2025-2026")
            .await
            .unwrap();

        let participants = lister_participants(&pool, a.id, "2025-2026").await.unwrap();
        assert_eq!(participants.len(), 0);
    }
}
