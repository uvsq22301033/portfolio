use axum::{
    extract::{State, Form}, // Ajout de TypedHeader pour récupérer le cookie
    routing::{get, post},
    Router,
    response::{Html}, // Ajout de IntoResponse et Redirect pour gérer cookie + redirection
};
use sqlx::{prelude::FromRow, SqlitePool};
use serde::{Serialize, Deserialize};
use tower_http::services::ServeDir;
use axum::extract::{Multipart, DefaultBodyLimit};
use tower_cookies::{CookieManagerLayer,Cookies,Cookie};
use axum::response::Redirect;





#[derive(Deserialize)]

struct PasswordForm {
    password: String,
}

#[derive(Serialize, FromRow)]
struct Photo {
    id: i32,
    filename: String,
    description: String,
    category: String, // 🔹 nouvelle colonne
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
    .route("/photo_invite", get(tout_photos_invite))
    .route("/photo_invite/portrait", get(portrait_photos_invite))
    .route("/photo_invite/animaux", get(animaux_photos_invite))
    .route("/photo_invite/paysage", get(paysage_photos_invite))
    .route("/photo_admin", get(get_photos_admin))
    .route("/upload", post(upload_photo))
    .route("/delete", post(supp_photo))
    .with_state(db.clone())
    .nest_service("/images", images_service)
    .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
    .layer(CookieManagerLayer::new());

    // Define the address for the server and run the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}



async fn identification(cookies: Cookies) -> Html<String> {
    cookies.add(Cookie::new("is_admin", "false"));
    let html = r#"
        <html>
            <head>
                <!-- 🔹 CHANGEMENT : style global -->
                <style>
                    body {
                        font-family: Arial, sans-serif;
                        background-color: #fafafa;
                        text-align: center;
                        margin: 50px;
                    }
                    h1 {
                        color: #333;
                        margin-bottom: 30px;
                    }
                    button {
                        background-color: #007BFF;
                        border: none;
                        color: white;
                        padding: 10px 20px;
                        font-size: 16px;
                        border-radius: 6px;
                        margin: 5px;
                        cursor: pointer;
                        transition: background-color 0.3s;
                    }
                    button:hover {
                        background-color: #0056b3;
                    }
                    input[type="password"] {
                        padding: 10px;
                        font-size: 16px;
                        border-radius: 6px;
                        border: 1px solid #ccc;
                        margin: 5px 0;
                        width: 200px;
                    }
                    form {
                        display: inline-block; /* 🔹 centrer le formulaire */
                        margin-top: 20px;
                    }
                    .button-container {
                        margin-top: 20px;
                    }
                </style>
            </head>
            <body>
                <h1>IDENTIFICATION</h1>

                <div class="button-container">
                    <a href="/homepage_invite">
                        <button>Invité(e)</button>
                    </a>
                </div>

                <form action="/redirect" method="post">
                    <input type="password" name="password" placeholder="Mot de passe admin"/><br/>
                    <button type="submit">Entrer</button>
                </form>
            </body>
        </html>
    "#;
    Html(html.to_string())
}


#[axum::debug_handler]
async fn redirect(
    cookies: Cookies,
    Form(form): Form<PasswordForm>,
) -> Html<String> {

    if form.password == ADMIN_PASSWORD {
        cookies.add(Cookie::new("is_admin", "true"));

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

        // Retour du HTML avec cookie
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
            <head>
                <!-- 🔹 CHANGEMENT : ajout de style global -->
                <style>
                    body { 
                        font-family: Arial, sans-serif; 
                        background-color: #fafafa; 
                        text-align: center; 
                        margin: 40px;
                    }
                    h1 { color: #333; }
                    button {
                        background-color: #007BFF;
                        border: none;
                        color: white;
                        padding: 10px 20px;
                        text-align: center;
                        text-decoration: none;
                        font-size: 16px;
                        border-radius: 6px;
                        margin: 5px;
                        cursor: pointer;
                    }
                    button:hover {
                        background-color: #0056b3;
                    }
                    img {
                        border-radius: 8px;
                        box-shadow: 0 2px 6px rgba(0,0,0,0.2);
                        margin: 15px;
                    }
                    .button-container {  /* 🔹 CHANGEMENT : conteneur pour les boutons */
                        margin-top: 20px;
                        display: flex;
                        justify-content: center;
                        gap: 10px;  /* espace entre les boutons */
                    }
                </style>
            </head>
            <body>
                <h1>Bienvenue sur mon portfolio !</h1>
                <p style="font-size:18px; color:#555; margin-bottom:15px;">
                Bienvenue voyageur !
                </p>
                <p style="font-size:18px; color:#555; margin-bottom:15px;">
                    Ayant fait l'acquisition de l'appareil photo que voici, je vous invite à découvrir mes magnifiques créations.
                    Ce site a été créé à la main, et avec amour ❤️, donc au moindre problème, n'hésitez pas à me contacter.
                </p>
                <p style="font-size:18px; color:#0056b3; margin-bottom:15px;">
                    Bienvenue dans mon univers et bonne visite !
                </p>
                <img src="/images/banniere.jpg" alt="bannière" width="600"/>
                 
                <div class="button-container">
                    <a href="/photo_invite"><button>Voir les photos</button></a>
                    <a href="/"><button>Identification</button></a>
                </div>
            </body>
        </html>
    "#;
    Html(html.to_string())
}



async fn homepage_admin(cookies: Cookies) -> Html<String> {
    let is_admin = cookies.get("is_admin").map(|c| c.value() == "true").unwrap_or(false);
    if !is_admin {
        return Html("<h1>Accès refusé</h1><a href='/'><button>Retour</button></a>".to_string());
    }

    Html(r#"
        <html>
            <head>
                <style>
                    body {
                        font-family: Arial, sans-serif;
                        background-color: #fafafa;
                        text-align: center;
                        padding: 40px;
                    }
                    form {
                        background-color: #fff;
                        padding: 20px;
                        border-radius: 10px;
                        box-shadow: 0 2px 8px rgba(0,0,0,0.1);
                        display: inline-block;
                        margin-bottom: 20px;
                    }
                    input, select, button, textarea {
                        margin: 10px;
                        padding: 10px;
                        border-radius: 5px;
                        border: 1px solid #ccc;
                        width: 80%;
                        max-width: 300px;
                    }
                    button {
                        background-color: #007BFF;
                        color: white;
                        border: none;
                        cursor: pointer;
                    }
                    button:hover {
                        background-color: #0056b3;
                    }
                </style>
            </head>
            <body>
                <h1>Bienvenue Admin !</h1>
                
                <!-- 🔹 CHANGEMENT : formulaire complet avec description et catégorie -->
                <form action="/upload" method="post" enctype="multipart/form-data">
                    <input type="file" name="file" accept="image/*" required/><br>
                    <textarea name="description" placeholder="Description" rows="3"></textarea><br>
                    <select name="category" required>
                        <option value="paysage">Paysage</option>
                        <option value="portrait">Portrait</option>
                        <option value="animaux">Animaux</option>
                        <option value="autre">Autre</option>
                    </select><br>
                    <button type="submit">Uploader une image</button>
                </form>

                <a href="/photo_admin"><button>Voir les photos</button></a>
                <a href="/"><button>Déconnexion</button></a>
            </body>
        </html>
    "#.to_string())
}



async fn tout_photos_invite(
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {

    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT id, filename, description, category FROM photos"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
        <html>
            <head>
                <style>
                    body {
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                        background-color: #f0f2f5;
                        text-align: center;
                        margin: 0;
                        padding: 0;
                    }
                    h1 {
                        color: #222;
                        margin: 30px 0;
                        font-size: 2.5em;
                    }
                    .gallery {
                        display: flex;
                        flex-wrap: wrap;
                        justify-content: center;
                        gap: 20px;
                        padding: 20px;
                    }
                    .photo-card {
                        background: white;
                        box-shadow: 0 4px 15px rgba(0,0,0,0.2);
                        overflow: hidden;
                        width: 700px;
                        transition: transform 0.3s, box-shadow 0.3s;
                    }
                    .photo-card:hover {
                        transform: translateY(-5px);
                        box-shadow: 0 8px 25px rgba(0,0,0,0.3);
                    }
                    .photo-card img {
                        width: 100%;
                        height: auto;
                        display: block;
                        border-radius: 0;
                    }
                    .photo-card .desc {
                        padding: 15px;
                        text-align: left;
                    }
                    .photo-card .desc p {
                        margin: 5px 0;
                        color: #555;
                    }
                    .photo-card .desc span {
                        font-weight: bold;
                        color: #333;
                    }
                    .btn {
                        margin: 10px 5px;
                        background-color: #007BFF;
                        border: none;
                        color: white;
                        padding: 10px 20px;
                        font-size: 16px;
                        cursor: pointer;
                        text-decoration: none;
                        border-radius: 6px;
                        display: inline-block; /* 🔹 évite les chevauchements */
                    }
                    .btn:hover {
                        background-color: #0056b3;
                    }
                    /* 🔹 Conteneur boutons d’action */
                    .actions {
                        margin-bottom: 20px;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 10px; /* espace entre les lignes */
                    }
                    /* 🔹 Ligne spéciale pour filtres */
                    .filters {
                        display: flex;
                        justify-content: center;
                        flex-wrap: wrap;
                        gap: 10px;
                    }
                </style>
            </head>
            <body>
                <h1>Galerie</h1>

                <!-- 🔹 Nouvelle structure pour les boutons -->
                <div class='actions'>
                    <a class='btn' href='/homepage_invite'>Accueil</a>

                    <div class='filters'>
                        <a class='btn' href='/photo_invite/animaux'>Animaux</a>
                        <a class='btn' href='/photo_invite/portrait'>Portrait</a>
                        <a class='btn' href='/photo_invite/paysage'>Paysage</a>
                    </div>
                </div>

                <div class='gallery'>
    "#);

    for photo in rows {
        html.push_str(&format!(
            r#"
                <div class='photo-card'>
                    <img src='/images/{0}' alt='{1}'/>
                    <div class='desc'>
                        <p><span>Catégorie:</span> {1}</p>
                        <p><span>Description:</span> {2}</p>
                    </div>
                </div>
            "#,
            photo.filename,
            photo.category,
            photo.description
        ));
    }

    html.push_str("</div></body></html>");

    Ok(Html(html))
}





async fn portrait_photos_invite(
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {

    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT id, filename, description, category FROM photos WHERE category = 'portrait'"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
        <html>
            <head>
                <style>
                    body {
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                        background-color: #f0f2f5;
                        text-align: center;
                        margin: 0;
                        padding: 0;
                    }
                    h1 {
                        color: #222;
                        margin: 30px 0;
                        font-size: 2.5em;
                    }
                    .gallery {
                        display: flex;
                        flex-wrap: wrap;
                        justify-content: center;
                        gap: 20px;
                        padding: 20px;
                    }
                    .photo-card {
                        background: white;
                        box-shadow: 0 4px 15px rgba(0,0,0,0.2);
                        overflow: hidden;
                        width: 700px;
                        transition: transform 0.3s, box-shadow 0.3s;
                    }
                    .photo-card:hover {
                        transform: translateY(-5px);
                        box-shadow: 0 8px 25px rgba(0,0,0,0.3);
                    }
                    .photo-card img {
                        width: 100%;
                        height: auto;
                        display: block;
                        border-radius: 0;
                    }
                    .photo-card .desc {
                        padding: 15px;
                        text-align: left;
                    }
                    .photo-card .desc p {
                        margin: 5px 0;
                        color: #555;
                    }
                    .photo-card .desc span {
                        font-weight: bold;
                        color: #333;
                    }
                    .btn {
                        margin: 10px 5px;
                        background-color: #007BFF;
                        border: none;
                        color: white;
                        padding: 10px 20px;
                        font-size: 16px;
                        cursor: pointer;
                        text-decoration: none;
                        border-radius: 6px;
                        display: inline-block; /* 🔹 évite les chevauchements */
                    }
                    .btn:hover {
                        background-color: #0056b3;
                    }
                    /* 🔹 Conteneur boutons d’action */
                    .actions {
                        margin-bottom: 20px;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 10px; /* espace entre les lignes */
                    }
                    /* 🔹 Ligne spéciale pour filtres */
                    .filters {
                        display: flex;
                        justify-content: center;
                        flex-wrap: wrap;
                        gap: 10px;
                    }
                </style>
            </head>
            <body>
                <h1>Galerie</h1>

                <!-- 🔹 Nouvelle structure pour les boutons -->
                <div class='actions'>
                    <a class='btn' href='/homepage_invite'>Accueil</a>

                    <div class='filters'>
                        <a class='btn' href='/photo_invite'>Tous</a>
                        <a class='btn' href='/photo_invite/animaux'>Animaux</a>
                        <a class='btn' href='/photo_invite/paysage'>Paysage</a>
                    </div>
                </div>

                <div class='gallery'>
    "#);

    for photo in rows {
        html.push_str(&format!(
            r#"
                <div class='photo-card'>
                    <img src='/images/{0}' alt='{1}'/>
                    <div class='desc'>
                        <p><span>Catégorie:</span> {1}</p>
                        <p><span>Description:</span> {2}</p>
                    </div>
                </div>
            "#,
            photo.filename,
            photo.category,
            photo.description
        ));
    }

    html.push_str("</div></body></html>");

    Ok(Html(html))
}


async fn animaux_photos_invite(
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {

    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT id, filename, description, category FROM photos WHERE category = 'animaux'"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
        <html>
            <head>
                <style>
                    body {
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                        background-color: #f0f2f5;
                        text-align: center;
                        margin: 0;
                        padding: 0;
                    }
                    h1 {
                        color: #222;
                        margin: 30px 0;
                        font-size: 2.5em;
                    }
                    .gallery {
                        display: flex;
                        flex-wrap: wrap;
                        justify-content: center;
                        gap: 20px;
                        padding: 20px;
                    }
                    .photo-card {
                        background: white;
                        box-shadow: 0 4px 15px rgba(0,0,0,0.2);
                        overflow: hidden;
                        width: 700px;
                        transition: transform 0.3s, box-shadow 0.3s;
                    }
                    .photo-card:hover {
                        transform: translateY(-5px);
                        box-shadow: 0 8px 25px rgba(0,0,0,0.3);
                    }
                    .photo-card img {
                        width: 100%;
                        height: auto;
                        display: block;
                        border-radius: 0;
                    }
                    .photo-card .desc {
                        padding: 15px;
                        text-align: left;
                    }
                    .photo-card .desc p {
                        margin: 5px 0;
                        color: #555;
                    }
                    .photo-card .desc span {
                        font-weight: bold;
                        color: #333;
                    }
                    .btn {
                        margin: 10px 5px;
                        background-color: #007BFF;
                        border: none;
                        color: white;
                        padding: 10px 20px;
                        font-size: 16px;
                        cursor: pointer;
                        text-decoration: none;
                        border-radius: 6px;
                        display: inline-block; /* 🔹 évite les chevauchements */
                    }
                    .btn:hover {
                        background-color: #0056b3;
                    }
                    /* 🔹 Conteneur boutons d’action */
                    .actions {
                        margin-bottom: 20px;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 10px; /* espace entre les lignes */
                    }
                    /* 🔹 Ligne spéciale pour filtres */
                    .filters {
                        display: flex;
                        justify-content: center;
                        flex-wrap: wrap;
                        gap: 10px;
                    }
                </style>
            </head>
            <body>
                <h1>Galerie</h1>

                <!-- 🔹 Nouvelle structure pour les boutons -->
                <div class='actions'>
                    <a class='btn' href='/homepage_invite'>Accueil</a>

                    <div class='filters'>
                        <a class='btn' href='/photo_invite'>Tous</a>
                        <a class='btn' href='/photo_invite/portrait'>Portrait</a>
                        <a class='btn' href='/photo_invite/paysage'>Paysage</a>
                    </div>
                </div>

                <div class='gallery'>
    "#);

    for photo in rows {
        html.push_str(&format!(
            r#"
                <div class='photo-card'>
                    <img src='/images/{0}' alt='{1}'/>
                    <div class='desc'>
                        <p><span>Catégorie:</span> {1}</p>
                        <p><span>Description:</span> {2}</p>
                    </div>
                </div>
            "#,
            photo.filename,
            photo.category,
            photo.description
        ));
    }

    html.push_str("</div></body></html>");

    Ok(Html(html))
}

async fn paysage_photos_invite(
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {

    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT id, filename, description, category FROM photos WHERE category = 'paysage'"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
        <html>
            <head>
                <style>
                    body {
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                        background-color: #f0f2f5;
                        text-align: center;
                        margin: 0;
                        padding: 0;
                    }
                    h1 {
                        color: #222;
                        margin: 30px 0;
                        font-size: 2.5em;
                    }
                    .gallery {
                        display: flex;
                        flex-wrap: wrap;
                        justify-content: center;
                        gap: 20px;
                        padding: 20px;
                    }
                    .photo-card {
                        background: white;
                        box-shadow: 0 4px 15px rgba(0,0,0,0.2);
                        overflow: hidden;
                        width: 700px;
                        transition: transform 0.3s, box-shadow 0.3s;
                    }
                    .photo-card:hover {
                        transform: translateY(-5px);
                        box-shadow: 0 8px 25px rgba(0,0,0,0.3);
                    }
                    .photo-card img {
                        width: 100%;
                        height: auto;
                        display: block;
                        border-radius: 0;
                    }
                    .photo-card .desc {
                        padding: 15px;
                        text-align: left;
                    }
                    .photo-card .desc p {
                        margin: 5px 0;
                        color: #555;
                    }
                    .photo-card .desc span {
                        font-weight: bold;
                        color: #333;
                    }
                    .btn {
                        margin: 10px 5px;
                        background-color: #007BFF;
                        border: none;
                        color: white;
                        padding: 10px 20px;
                        font-size: 16px;
                        cursor: pointer;
                        text-decoration: none;
                        border-radius: 6px;
                        display: inline-block; /* 🔹 évite les chevauchements */
                    }
                    .btn:hover {
                        background-color: #0056b3;
                    }
                    /* 🔹 Conteneur boutons d’action */
                    .actions {
                        margin-bottom: 20px;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 10px; /* espace entre les lignes */
                    }
                    /* 🔹 Ligne spéciale pour filtres */
                    .filters {
                        display: flex;
                        justify-content: center;
                        flex-wrap: wrap;
                        gap: 10px;
                    }
                </style>
            </head>
            <body>
                <h1>Galerie</h1>

                <!-- 🔹 Nouvelle structure pour les boutons -->
                <div class='actions'>
                    <a class='btn' href='/homepage_invite'>Accueil</a>

                    <div class='filters'>
                        <a class='btn' href='/photo_invite'>Tous</a>
                        <a class='btn' href='/photo_invite/animaux'>Animaux</a>
                        <a class='btn' href='/photo_invite/portrait'>Portrait</a>
                    </div>
                </div>

                <div class='gallery'>
    "#);

    for photo in rows {
        html.push_str(&format!(
            r#"
                <div class='photo-card'>
                    <img src='/images/{0}' alt='{1}'/>
                    <div class='desc'>
                        <p><span>Catégorie:</span> {1}</p>
                        <p><span>Description:</span> {2}</p>
                    </div>
                </div>
            "#,
            photo.filename,
            photo.category,
            photo.description
        ));
    }

    html.push_str("</div></body></html>");

    Ok(Html(html))
}


async fn get_photos_admin(
    cookies: Cookies,
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {
    let is_admin = cookies.get("is_admin").map(|c| c.value() == "true").unwrap_or(false);
    if !is_admin {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }

    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT id, filename, description, category FROM photos"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
    <html>
        <body>
            <h1>Photos</h1>
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

                <form action="/delete" method="post">
                    <input type="hidden" name="id" value="{2}" />
                    <input type="hidden" name="filename" value="{0}" />
                    <button type="submit">Supprimer</button>
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
    cookies: Cookies,
    State(db): State<SqlitePool>,
    mut multipart: Multipart,
) -> Result<Redirect, String> {

    let is_admin = cookies.get("is_admin").map(|c| c.value() == "true").unwrap_or(false);
    if !is_admin {
        return Err("Accès refusé".to_string());
    }

    let mut filename = String::new();
    let mut description = String::new();
    let mut category = "autre".to_string(); // valeur par défaut

    while let Some(field) = multipart.next_field().await.map_err(|e| e.to_string())? {
        match field.name() {
            Some("file") => {
                filename = field.file_name().unwrap_or("file.jpg").to_string();
                let data = field.bytes().await.map_err(|e| e.to_string())?;
                let filepath = format!("images/{}", filename);
                tokio::fs::write(&filepath, &data).await.map_err(|e| e.to_string())?;
            },
            Some("description") => {
                description = String::from_utf8(field.bytes().await.map_err(|e| e.to_string())?.to_vec()).unwrap_or_default();
            },
            Some("category") => {
                category = String::from_utf8(field.bytes().await.map_err(|e| e.to_string())?.to_vec()).unwrap_or("autre".to_string());
            },
            _ => {}
        }
    }

    // Insert dans la BDD
    sqlx::query(
        "INSERT INTO photos (filename, description, category) VALUES (?, ?, ?)",
    )
    .bind(&filename)
    .bind(&description)
    .bind(&category)
    .execute(&db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Redirect::to("/photo_admin"))
}




async fn supp_photo(
    cookies: Cookies,
    State(db): State<SqlitePool>,
    Form(payload): Form<DeletePhoto>,
) -> Result<Redirect, String> {

    let is_admin = cookies.get("is_admin").map(|c| c.value() == "true").unwrap_or(false);
    if !is_admin {
        return Err("Accès refusé".to_string());
    }
    sqlx::query("DELETE FROM photos WHERE id = ?")
        .bind(payload.id)
        .execute(&db)
        .await
        .map_err(|e| e.to_string())?;
    let filepath = format!("images/{}", payload.filename);
    if tokio::fs::remove_file(&filepath).await.is_err() {
        return Err("Erreur lors de la suppression du fichier".to_string());
    }
    Ok(Redirect::to("/homepage_admin"))
}  