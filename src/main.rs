use axum::{
    extract::State, extract::Form, routing::{get, post}, Json, Router
};
use sqlx::{prelude::FromRow, SqlitePool};
use serde::Serialize;
use serde::Deserialize;
use tower_http::services::ServeDir;
use axum::extract::Multipart;
use axum::extract::DefaultBodyLimit;
use axum::response::Html;


#[derive(Deserialize)]

struct PasswordForm {
    password: String,
}

#[derive(Serialize, FromRow)]
struct Photo {
    id: i32,
    filename: String,
    description: String,
    created_at: String,
}

#[derive(Deserialize)]
struct DeletePhoto {
    id: i32,
    filename: String,
}


const ADMIN_PASSWORD: &str = "sinj";

#[tokio::main]
async fn main() {
    // initialisation de la db sqlite     
    let db = SqlitePool::connect("sqlite:portfolio.db")
        .await
        .expect("Impossible de se connecter à la base");

    let images_service = ServeDir::new("images");
    

    let app = Router::new()
    .route("/", get(identification))
    .route("/redirect", post(redirect))
    .route("/homepage_admin", get(homepage_admin))
    .route("/homepage_invite", get(homepage_invite))
    .route("/photo_invite", get(get_photos_invite))
    .route("/photo_admin", get(get_photos_admin))
    .route("/upload", post(upload_photo))
    .route("/delete", post(supp_photo))
    .with_state(db.clone())
    .nest_service("/images", images_service)
    .layer(DefaultBodyLimit::max(50 * 1024 * 1024));

    // Define the address for the server and run the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn identification() -> Html<String> {
    let html = r#"
         <html>
            <body>
                <h1>Bienvenue sur le portfolio</h1>
                <a href="/homepage_invite">
                    <button>invité(e)</button>
                </a>
                <form action="/redirect" method="post">
                    <input type="password" name="password" placeholder="Mot de passe admin"/>
                    <button type="submit">Entrer</button>
                </form>
            </body>
        </html>
    "#;
    Html(html.to_string())
}

async fn redirect(Form(form): Form<PasswordForm>) -> Html<String>{
    if form.password == ADMIN_PASSWORD {
    let html = r#"
         <html>
            <body>
                <h1>Authentification admin reussie</h1>
                <a href="/homepage_admin">
                    <button>continuer(e)</button>
                </a>
            </body>
        </html>
    "#;
    Html(html.to_string())
    } else {
        Html(r#"
            <html>
                <body>
                    <h1>Mot de passe incorrect</h1>
                    <a href="/"><button>Retour</button></a>
                </body>
            </html>
        "#.to_string())
    }
}


async fn homepage_invite() -> Html<String> {
    let html = r#"
         <html>
            <body>
                <h1>Welcome to the Portfolio Homepage!</h1>
                <!-- Bouton pour aller sur la galerie -->
                <a href="/photo_invite">
                    <button>Voir les photos</button>
                
                </a>
                <a href="/">
                    <button>identification</button>
                </a>

                <hr/>

                
    "#;
    Html(html.to_string())
}

async fn homepage_admin() -> Html<String> {

        Html(r#"
            <html>
                <body>
                    <h1>Bienvenue Admin !</h1>
                    <form action="/upload" method="post" enctype="multipart/form-data">
                        <input type="file" name="file"/>
                        <button type="submit">Uploader une image</button>
                    </form>
                    <a href="/photo_admin"><button>Voir les photos</button></a>
                    <a href="/"><button>Déconnexion</button></a>
                </body>
            </html>
        "#.to_string())
}



async fn get_photos_invite(
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {

    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT id, filename, description, created_at FROM photos"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
        <html>
            <body>
                <h1>Photos</h1>
                <!-- Bouton retour à l'accueil -->
                <a href="/homepage_invite">
                    <button>Accueil</button>
                </a>
    "#);
    for photo in rows {
            html.push_str(&format!(
                r#"
                <div>
                    <img src="/images/{0}" width="300" /><br/>
                    <p>{1}</p>
                </div>
                <hr/>
                "#,
                photo.filename, photo.description
            ));}
    html.push_str("</body></html>");

    Ok(Html(html))
}


async fn get_photos_admin(
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {

    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT id, filename, description, created_at FROM photos"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
    <html>
        <body>
            <h1>Photos</h1>
            <!-- Bouton retour à l'accueil -->
            <form action="/homepage_admin">
                <button>Accueil</button>
            </form>
    "#);

    for photo in rows {
        html.push_str(&format!(
            r#"
            <div>
                <img src="/images/{0}" width="300" /><br/>
                <p>{1}</p>
            </div>
            <hr/>
                    <p>{1}</p>

                    <!-- FORMULAIRE SUPPRIMER -->
                    <form action="/delete" method="post">
                        <!-- On envoie l'id et le nom du fichier -->
                        <input type="hidden" name="id" value="{2}" />
                        <input type="hidden" name="filename" value="{0}" />
                        <button type="submit">Supprimer</button> <!-- bouton pour supprimer la photo -->
                    </form>
                </div>
                <hr/>
                "#,
                photo.filename, photo.description, photo.id
            ));
    }
    html.push_str("</body></html>");

    Ok(Html(html))
}



async fn upload_photo(
    State(db): State<SqlitePool>,
    mut multipart: Multipart,
) -> Result<Json<Photo>, String> {

    // On récupère le premier champ du multipart (le fichier)
    while let Some(field) = multipart.next_field().await.map_err(|e| e.to_string())? {
        let filename = field.file_name().unwrap_or("file.jpg").to_string();
        let data = field.bytes().await.map_err(|e| e.to_string())?;



        // Sauvegarde locale
        let filepath = format!("images/{}", filename);
        tokio::fs::write(&filepath, &data)
        .await
        .map_err(|e| e.to_string())?;

        // Insert dans la BDD
        let result = sqlx::query(
            "INSERT INTO photos (filename) VALUES (?)",
        )
        .bind(filename)
        .execute(&db)
        .await
        .map_err(|e| e.to_string())?;

        let id = result.last_insert_rowid();

        let photo = sqlx::query_as::<_, Photo>(
            "SELECT id, filename, description, created_at FROM photos WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&db)
        .await
        .map_err(|e| e.to_string())?;

        return Ok(Json(photo));
    }

    Err("Aucun fichier reçu".to_string())
}

async fn supp_photo(
    State(db): State<SqlitePool>,
    Form(payload): Form<DeletePhoto>,
) -> Result<Html<String>, String> {
    if true {
    sqlx::query("DELETE FROM photos WHERE id = ?")
        .bind(payload.id)
        .execute(&db)
        .await
        .map_err(|e| e.to_string())?;
    let filepath = format!("images/{}", payload.filename);
    if tokio::fs::remove_file(&filepath).await.is_err() {
        return Err("Erreur lors de la suppression du fichier".to_string());
    }
    return Ok(Html("<script>window.location='/photo_admin'</script>".to_string()));
    }  
    else {
        Err("Suppression non autorisée".to_string())
    }
    
}

