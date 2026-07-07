use sqlx::SqlitePool;

use crate::domain::personne::{
    CreatePersonne, CriteresRecherchePersonnes, Pagination, Personne, ResultatRecherchePersonnes,
    UpdatePersonne,
};

pub async fn create(pool: &SqlitePool, input: CreatePersonne) -> Result<Personne, sqlx::Error> {
    let row = sqlx::query_as::<_, Personne>(
        "INSERT INTO personnes_physiques (nom, prenom, date_naissance, email, telephone, responsable_id)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING *",
    )
    .bind(&input.nom)
    .bind(&input.prenom)
    .bind(input.date_naissance)
    .bind(&input.email)
    .bind(&input.telephone)
    .bind(input.responsable_id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn update(
    pool: &SqlitePool,
    id: i64,
    input: UpdatePersonne,
) -> Result<Personne, sqlx::Error> {
    let row = sqlx::query_as::<_, Personne>(
        "UPDATE personnes_physiques
         SET nom = ?, prenom = ?, date_naissance = ?, email = ?, telephone = ?, responsable_id = ?
         WHERE id = ?
         RETURNING *",
    )
    .bind(&input.nom)
    .bind(&input.prenom)
    .bind(input.date_naissance)
    .bind(&input.email)
    .bind(&input.telephone)
    .bind(input.responsable_id)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Personne>, sqlx::Error> {
    let row = sqlx::query_as::<_, Personne>("SELECT * FROM personnes_physiques WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

pub async fn rechercher(
    pool: &SqlitePool,
    criteres: CriteresRecherchePersonnes,
    pagination: Pagination,
) -> Result<ResultatRecherchePersonnes, sqlx::Error> {
    let annee_scolaire = crate::domain::personne::current_annee_scolaire();
    let pattern = criteres.texte_libre.as_ref().map(|t| format!("%{}%", t));

    let mut conditions: Vec<String> = Vec::new();

    if criteres.texte_libre.is_some() {
        let cols = ["pp.nom", "pp.prenom", "pp.email", "pp.telephone"];
        let ors: Vec<String> = cols
            .iter()
            .map(|c| format!("LOWER({}) LIKE LOWER(?)", c))
            .collect();
        conditions.push(format!("({})", ors.join(" OR ")));
    }

    if criteres.adherent_uniquement {
        conditions.push(
            "EXISTS (SELECT 1 FROM adhesions a WHERE a.personne_id = pp.id AND a.annee_scolaire = ?)"
                .to_string(),
        );
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE 1=1 AND {}", conditions.join(" AND "))
    };

    // --- count ---
    let count_sql = format!(
        "SELECT COUNT(*) FROM personnes_physiques pp{}",
        where_clause
    );

    let mut count_query = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_sql.as_str()));

    if let Some(ref p) = pattern {
        count_query = count_query.bind(p).bind(p).bind(p).bind(p);
    }
    if criteres.adherent_uniquement {
        count_query = count_query.bind(&annee_scolaire);
    }

    let total: i64 = count_query.fetch_one(pool).await?;

    // --- data ---
    let offset = if pagination.par_page > 0 {
        (pagination.page - 1) * pagination.par_page
    } else {
        0
    };

    let data_sql = if pagination.par_page > 0 {
        format!(
            "SELECT pp.* FROM personnes_physiques pp{} ORDER BY pp.nom, pp.prenom LIMIT ? OFFSET ?",
            where_clause
        )
    } else {
        format!(
            "SELECT pp.* FROM personnes_physiques pp{} ORDER BY pp.nom, pp.prenom",
            where_clause
        )
    };

    let mut data_query = sqlx::query_as::<_, Personne>(sqlx::AssertSqlSafe(data_sql.as_str()));

    if let Some(ref p) = pattern {
        data_query = data_query.bind(p).bind(p).bind(p).bind(p);
    }
    if criteres.adherent_uniquement {
        data_query = data_query.bind(&annee_scolaire);
    }
    if pagination.par_page > 0 {
        data_query = data_query.bind(pagination.par_page).bind(offset);
    }

    let donnees = data_query.fetch_all(pool).await?;

    let pages = if pagination.par_page > 0 {
        (total as f64 / pagination.par_page as f64).ceil() as u32
    } else {
        1
    };

    Ok(ResultatRecherchePersonnes {
        donnees,
        total: total as u32,
        page: pagination.page,
        pages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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

    async fn seed_personne(
        pool: &SqlitePool,
        nom: &str,
        prenom: &str,
        email: Option<&str>,
        telephone: Option<&str>,
    ) -> Personne {
        sqlx::query_as::<_, Personne>(
            "INSERT INTO personnes_physiques (nom, prenom, date_naissance, email, telephone)
             VALUES (?, ?, ?, ?, ?) RETURNING *",
        )
        .bind(nom)
        .bind(prenom)
        .bind("2000-01-15")
        .bind(email)
        .bind(telephone)
        .fetch_one(pool)
        .await
        .expect("failed to seed personne")
    }

    async fn seed_adhesion(pool: &SqlitePool, personne_id: i64, annee_scolaire: &str) {
        sqlx::query(
            "INSERT INTO adhesions (personne_id, annee_scolaire, reglee)
             VALUES (?, ?, 1)",
        )
        .bind(personne_id)
        .bind(annee_scolaire)
        .execute(pool)
        .await
        .expect("failed to seed adhesion");
    }

    #[tokio::test]
    async fn test_texte_libre_cherche_nom() {
        let pool = setup_db().await;
        seed_personne(&pool, "Dupont", "Jean", None, None).await;
        seed_personne(&pool, "Martin", "Alice", None, None).await;

        let resultat = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: Some("dup".into()),
                adherent_uniquement: false,
            },
            Pagination {
                page: 1,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat.total, 1);
        assert_eq!(resultat.donnees.len(), 1);
        assert_eq!(resultat.donnees[0].nom, "Dupont");
    }

    #[tokio::test]
    async fn test_texte_libre_cherche_prenom() {
        let pool = setup_db().await;
        seed_personne(&pool, "Dupont", "Jean", None, None).await;
        seed_personne(&pool, "Martin", "Jeanne", None, None).await;
        seed_personne(&pool, "Durand", "Pierre", None, None).await;

        let resultat = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: Some("jean".into()),
                adherent_uniquement: false,
            },
            Pagination {
                page: 1,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat.total, 2);
    }

    #[tokio::test]
    async fn test_texte_libre_cherche_email() {
        let pool = setup_db().await;
        seed_personne(&pool, "Dupont", "Jean", Some("jean@example.com"), None).await;
        seed_personne(&pool, "Martin", "Alice", Some("alice@gmail.com"), None).await;

        let resultat = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: Some("gmail".into()),
                adherent_uniquement: false,
            },
            Pagination {
                page: 1,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat.total, 1);
        assert_eq!(resultat.donnees[0].nom, "Martin");
    }

    #[tokio::test]
    async fn test_texte_libre_cherche_telephone() {
        let pool = setup_db().await;
        seed_personne(&pool, "Dupont", "Jean", None, Some("0612345678")).await;
        seed_personne(&pool, "Martin", "Alice", None, Some("0798765432")).await;

        let resultat = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: Some("0612".into()),
                adherent_uniquement: false,
            },
            Pagination {
                page: 1,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat.total, 1);
        assert_eq!(resultat.donnees[0].nom, "Dupont");
    }

    #[tokio::test]
    async fn test_sans_criteres() {
        let pool = setup_db().await;
        seed_personne(&pool, "C", "X", None, None).await;
        seed_personne(&pool, "A", "Y", None, None).await;
        seed_personne(&pool, "B", "Z", None, None).await;

        let resultat = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: None,
                adherent_uniquement: false,
            },
            Pagination {
                page: 1,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat.total, 3);
        assert_eq!(resultat.donnees.len(), 3);
        assert_eq!(resultat.pages, 1);
    }

    #[tokio::test]
    async fn test_aucun_resultat() {
        let pool = setup_db().await;
        seed_personne(&pool, "Dupont", "Jean", None, None).await;

        let resultat = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: Some("xyzzzzz".into()),
                adherent_uniquement: false,
            },
            Pagination {
                page: 1,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat.total, 0);
        assert_eq!(resultat.donnees.len(), 0);
        assert_eq!(resultat.pages, 0);
    }

    #[tokio::test]
    async fn test_pagination_page_1() {
        let pool = setup_db().await;
        for i in 0..25 {
            seed_personne(&pool, &format!("Nom{:02}", i), "Prenom", None, None).await;
        }

        let resultat = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: None,
                adherent_uniquement: false,
            },
            Pagination {
                page: 1,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat.total, 25);
        assert_eq!(resultat.donnees.len(), 20);
        assert_eq!(resultat.page, 1);
        assert_eq!(resultat.pages, 2);
    }

    #[tokio::test]
    async fn test_pagination_page_2() {
        let pool = setup_db().await;
        for i in 0..25 {
            seed_personne(&pool, &format!("Nom{:02}", i), "Prenom", None, None).await;
        }

        let resultat = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: None,
                adherent_uniquement: false,
            },
            Pagination {
                page: 2,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat.total, 25);
        assert_eq!(resultat.donnees.len(), 5);
        assert_eq!(resultat.page, 2);
        assert_eq!(resultat.pages, 2);
    }

    #[tokio::test]
    async fn test_pagination_par_page_0() {
        let pool = setup_db().await;
        for i in 0..25 {
            seed_personne(&pool, &format!("Nom{:02}", i), "Prenom", None, None).await;
        }

        let resultat = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: None,
                adherent_uniquement: false,
            },
            Pagination {
                page: 1,
                par_page: 0,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat.total, 25);
        assert_eq!(resultat.donnees.len(), 25);
        assert_eq!(resultat.pages, 1);
    }

    #[tokio::test]
    async fn test_adherent_uniquement() {
        let pool = setup_db().await;
        let p1 = seed_personne(&pool, "Dupont", "Jean", None, None).await;
        let _p2 = seed_personne(&pool, "Martin", "Alice", None, None).await;
        let p3 = seed_personne(&pool, "Durand", "Pierre", None, None).await;

        let annee = crate::domain::personne::current_annee_scolaire();
        seed_adhesion(&pool, p1.id, &annee).await;
        seed_adhesion(&pool, p3.id, &annee).await;

        let resultat = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: None,
                adherent_uniquement: true,
            },
            Pagination {
                page: 1,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat.total, 2);
        assert_eq!(resultat.donnees.len(), 2);
    }

    #[tokio::test]
    async fn test_texte_libre_et_adherent() {
        let pool = setup_db().await;
        let p1 = seed_personne(&pool, "Dupont", "Jean", None, None).await;
        let _p2 = seed_personne(&pool, "Dupond", "Alice", None, None).await;

        let annee = crate::domain::personne::current_annee_scolaire();
        seed_adhesion(&pool, p1.id, &annee).await;

        let resultat = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: Some("dup".into()),
                adherent_uniquement: true,
            },
            Pagination {
                page: 1,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat.total, 1);
        assert_eq!(resultat.donnees[0].nom, "Dupont");
    }

    #[tokio::test]
    async fn test_casse_insensible() {
        let pool = setup_db().await;
        seed_personne(&pool, "Dupont", "Jean", None, None).await;

        let resultat_min = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: Some("dup".into()),
                adherent_uniquement: false,
            },
            Pagination {
                page: 1,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        let resultat_maj = rechercher(
            &pool,
            CriteresRecherchePersonnes {
                texte_libre: Some("DUP".into()),
                adherent_uniquement: false,
            },
            Pagination {
                page: 1,
                par_page: 20,
            },
        )
        .await
        .unwrap();

        assert_eq!(resultat_min.total, 1);
        assert_eq!(resultat_maj.total, 1);
    }
}
