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
use tower_cookies::{CookieManagerLayer,Cookies,Cookie, Key};
use tower_cookies::cookie::{SameSite, time::Duration};
use axum::response::Redirect;
use sha2::{Sha256, Digest};
use std::{env, path::Path};



#[derive(Deserialize)]

struct PasswordForm {
    password: String,
}

#[derive(Serialize, FromRow)]
struct Photo {
    filename: String,
    description: String,
    category: String, // 🔹 nouvelle colonne
}

#[derive(Deserialize)]
struct DeletePhoto {
    filename: String,
}


const ADMIN_SESSION_COOKIE: &str = "admin_session";

fn env_value(name: &str) -> Option<String> {
    env::var(name).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn admin_password() -> Option<String> {
    env_value("ADMIN_PASSWORD")
}

fn cookie_secure() -> bool {
    env_value("ADMIN_COOKIE_SECURE").map(|value| value != "false").unwrap_or(true)
}

fn session_key() -> Option<Key> {
    let secret = env_value("SESSION_SECRET")?;
    if secret.len() < 32 {
        return None;
    }

    let mut first = Sha256::new();
    first.update(b"portfolio-admin-session-key-1:");
    first.update(secret.as_bytes());

    let mut second = Sha256::new();
    second.update(b"portfolio-admin-session-key-2:");
    second.update(secret.as_bytes());

    let mut key_material = Vec::with_capacity(64);
    key_material.extend_from_slice(&first.finalize());
    key_material.extend_from_slice(&second.finalize());
    Some(Key::from(&key_material))
}

fn admin_session_cookie() -> Cookie<'static> {
    Cookie::build((ADMIN_SESSION_COOKIE, "true"))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(cookie_secure())
        .max_age(Duration::hours(8))
        .build()
}

fn remove_admin_session_cookie() -> Cookie<'static> {
    Cookie::build(ADMIN_SESSION_COOKIE).path("/").build()
}

fn is_admin(cookies: &Cookies) -> bool {
    session_key()
        .and_then(|key| cookies.signed(&key).get(ADMIN_SESSION_COOKIE))
        .map(|cookie| cookie.value() == "true")
        .unwrap_or(false)
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn safe_filename(filename: &str) -> Option<String> {
    let filename = filename.trim();
    let basename = Path::new(filename).file_name()?.to_str()?;
    if filename != basename || basename.is_empty() || basename == "." || basename == ".." {
        return None;
    }

    let has_valid_chars = basename
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
    if !has_valid_chars {
        return None;
    }

    let extension = Path::new(basename).extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" => Some(basename.to_string()),
        _ => None,
    }
}

#[tokio::main]
async fn main() {
    // initialisation de la db sqlite     
    let db = SqlitePool::connect("sqlite:portfolio.db")
        .await
        .expect("Impossible de se connecter à la base");

    let images_service = ServeDir::new("images");
    

    let app = Router::new()
    .route("/", get(homepage_invite))
    .route("/identification", get(identification))
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
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}



async fn identification(cookies: Cookies) -> Html<String> {
    if let Some(key) = session_key() {
        cookies.signed(&key).remove(remove_admin_session_cookie());
    }
    let html = r#"
        <html>
            <head>
                <!-- 🔹 CHANGEMENT : style global et responsive -->
                <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <style>
                    body {
                        font-family: Arial, sans-serif;
                        background-color: #d8d8d0;
                        text-align: center;
                        margin: 20px;
                        padding: 0;
                    }
                    h1 {
                        color: #333;
                        margin-bottom: 30px;
                    }
                    button {
                        background-color: #d8d8d3;
                        border: none;
                        color: black;
                        padding: 0.8em 1.5em;
                        font-size: 0.9em;
                        border-radius: 4px;
                        margin: 5px;
                        cursor: pointer;
                        transition: background-color 0.3s;
                    }
                    button:hover {
                        background-color: #c6c6c0;
                    }
                    input[type="password"] {
                        padding: 10px;
                        font-size: 16px;
                        border-radius: 6px;
                        border: 1px solid #ccc;
                        margin: 5px 0;
                        width: 80%;
                        max-width: 250px;
                    }
                    form {
                        display: inline-block; /* 🔹 centre le formulaire */
                        margin-top: 20px;
                    }
                    .button-container {
                        margin-bottom: 20px;
                    }
                    @media (max-width: 500px) {
                        button {
                            width: 80%; /* 🔹 boutons adaptatifs sur mobile */
                        }
                        input[type="password"] {
                            width: 80%; /* 🔹 input adaptatif */
                        }
                    }
                </style>
            </head>
            <body>
                <h1>IDENTIFICATION</h1>

                <div class="button-container">
                    <a href="/">
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

    let Some(password) = admin_password() else {
        return Html(r#"
            <html>
                <body>
                    <h1>Configuration admin manquante</h1>
                    <p>Définissez ADMIN_PASSWORD et SESSION_SECRET côté serveur.</p>
                    <a href="/identification"><button>Retour</button></a>
                </body>
            </html>
        "#.to_string());
    };

    let Some(key) = session_key() else {
        return Html(r#"
            <html>
                <body>
                    <h1>Configuration session manquante</h1>
                    <p>SESSION_SECRET doit contenir au moins 32 caractères.</p>
                    <a href="/identification"><button>Retour</button></a>
                </body>
            </html>
        "#.to_string());
    };

    if form.password == password {
        cookies.signed(&key).add(admin_session_cookie());

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
                    <a href="/identification"><button>Retour</button></a>
                </body>
            </html>
        "#.to_string())
    }
}


async fn homepage_invite() -> Html<String> {
    let html = r#"
        <html>
            <head>
                <!-- 🔹 CHANGEMENT : style global et responsive -->
                <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <style>
                    body { 
                        font-family: Arial, sans-serif; 
                        background-color: #d8d8d0; 
                        text-align: center; 
                        margin: 20px;
                        padding: 0;
                    }
                    h1 { 
                        color: #000000ff; 
                        margin-bottom: 20px;
                    }
                    p {
                        font-size: 1em;
                        color: #000000ff;
                        margin: 10px auto;
                        max-width: 600px;
                        line-height: 1.5;
                    }
                    img {
                        width: 80%; /* 🔹 image responsive */
                        max-width: 600px;
                        margin: 15px 0;
                        box-shadow: 0 8px 24px rgba(0,0,0,0.08);
                    }
                    .button-container {
                        margin-top: 20px;
                        display: flex;
                        justify-content: center;
                        gap: 10px;
                        flex-wrap: wrap; /* 🔹 pour que les boutons passent à la ligne sur mobile */
                    }
                    button {
                        background-color: #d8d8d0;
                        border: none;
                        color: black;
                        padding: 0.8em 1.5em;
                        font-size: 1em;
                        border-radius: 4px;
                        cursor: pointer;
                        transition: background-color 0.3s;
                    }
                    button:hover {
                        background-color: #c6c6c0;
                    }
                    @media (max-width: 500px) {
                        button {
                            width: 80%; /* 🔹 boutons adaptatifs sur mobile */
                        }
                        img {
                            width: 95%; /* 🔹 image presque pleine largeur sur mobile */
                        }
                    }
                </style>
            </head>
            <body>
                <h1>Bienvenue sur mon portfolio !</h1>
                <p>Bienvenue voyageur !</p>
                <p>
                    Ayant fait l'acquisition de l'appareil photo que voici, je vous invite à découvrir mes magnifiques créations.
                    Ce site a été créé à la main, et avec amour ❤️, donc au moindre problème, n'hésitez pas à me contacter.
                </p>
                <p style="color: #666;">Pour profiter pleinement, montez la luminosité de votre écran.</p>
                <p style="color: #666;">Bienvenue et bonne visite !</p>
                <img src="/images/banniere.jpg" alt="bannière"/>
                 
                <div class="button-container">
                    <a href="/photo_invite"><button>Voir les photos</button></a>
                    <a href="/identification"><button>Identification</button></a>
                </div>
            </body>
        </html>
    "#;
    Html(html.to_string())
}



async fn homepage_admin(cookies: Cookies) -> Html<String> {
    if !is_admin(&cookies) {
        return Html("<h1>Accès refusé</h1><a href='/identification'><button>Retour</button></a>".to_string());
    }

    Html(r#"
        <html>
            <head>
                <style>
                    body {
                        font-family: Arial, sans-serif;
                        background-color: #d8d8d0;
                        text-align: center;
                        padding: 40px;
                    }
                    form {
                        background-color: #ffffff;
                        padding: 20px;
                        border-radius: 6px;
                        box-shadow: 0 10px 28px rgba(0,0,0,0.08);
                        display: inline-block;
                        margin-bottom: 20px;
                    }
                    input, select, button, textarea {
                        margin: 10px;
                        padding: 10px;
                        border-radius: 4px;
                        border: 1px solid #ccc;
                        width: 80%;
                        max-width: 300px;
                    }
                    button {
                        background-color: #d8d8d0;
                        color: #222;
                        border: none;
                        cursor: pointer;
                    }
                    button:hover {
                        background-color: #c6c6c0;
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
                <a href="/identification"><button>Déconnexion</button></a>
            </body>
        </html>
    "#.to_string())
}



async fn tout_photos_invite(
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {
    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT filename, description, category FROM photos"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
        <html>
            <head>
                <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <style>
                    body {
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                        background-color: #d8d8d0;
                        text-align: center;
                        margin: 0;
                        padding: 0;
                    }
                    h1 {
                        color: #222;
                        margin: 30px 0;
                        font-size: 2em;
                    }
                    .gallery {
                        display: grid;
                        grid-template-columns: minmax(0, 1fr);
                        gap: 22px;
                        padding: 20px;
                        max-width: 780px;
                        margin: 0 auto;
                    }
                    .photo-card {
                        background: #dcddd5;
                        border: 1px solid #d1d1c8;
                        box-shadow: 0 6px 18px rgba(0,0,0,0.035);
                        overflow: hidden;
                        width: 100%;
                        max-width: none;
                        transition: transform 0.3s, box-shadow 0.3s;
                        cursor: pointer;
                    }
                    .photo-card:hover {
                        transform: translateY(-5px);
                        box-shadow: 0 10px 24px rgba(0,0,0,0.07);
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
                        color: #666;
                    }
                    .photo-card .desc span {
                        font-weight: bold;
                        color: #333;
                    }
                    .btn {
                        margin: 5px;
                        background-color: #d8d8d0;
                        border: none;
                        color: #222;
                        padding: 0.8em 1.5em;
                        font-size: 1em;
                        cursor: pointer;
                        text-decoration: none;
                        border-radius: 4px;
                        display: inline-block;
                        transition: background-color 0.3s;
                    }
                    .btn:hover {
                        background-color: #c6c6c0;
                    }
                    .actions {
                        margin: 20px 0;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 10px;
                    }
                    .filters {
                        display: flex;
                        justify-content: center;
                        flex-wrap: wrap;
                        gap: 10px;
                    }
                    @media (max-width: 600px) {
                        .btn {
                            width: 80%;
                        }
                        .photo-card {
                            width: 95%;
                        }
                    }

                    /* 🔹 Agrandissement */
                    .overlay {
                        display: none;
                        position: fixed;
                        top: 0;
                        left: 0;
                        width: 100%;
                        height: 100%;
                        background: rgba(0, 0, 0, 0.8);
                        justify-content: center;
                        align-items: center;
                        z-index: 1000;
                    }
                    .overlay img {
                        max-width: 95%;
                        max-height: 90%;
                        border-radius: 10px;
                        box-shadow: 0 0 20px rgba(0, 0, 0, 1);
                    }
                </style>
            </head>
            <body>
                <h1>Galerie</h1>

                <div class='actions'>
                    <a class='btn' href='/'>Accueil</a>
                    <div class='filters'>
                        <a class='btn' href='/photo_invite/animaux'>Animaux</a>
                        <a class='btn' href='/photo_invite/portrait'>Portrait</a>
                        <a class='btn' href='/photo_invite/paysage'>Paysage</a>
                    </div>
                </div>

                <div class='gallery'>
    "#);

    for photo in rows {
        let filename = html_escape(&photo.filename);
        let category = html_escape(&photo.category);
        let description = html_escape(&photo.description);
        html.push_str(&format!(
            r#"
                <div class='photo-card' onclick="openImage('/images/{0}')">
                    <img src='/images/{0}' alt='{1}'/>
                    <div class='desc'>
                        <p><span>Catégorie:</span> {1}</p>
                        <p><span>Description:</span> {2}</p>
                    </div>
                </div>
            "#,
            filename,
            category,
            description
        ));
    }

    html.push_str(r#"
                </div>
                <div class="overlay" id="overlay" onclick="closeImage()">
                    <img id="overlay-img" src="" alt=""/>
                </div>
                <script>
                    function openImage(src) {
                        const overlay = document.getElementById('overlay');
                        const img = document.getElementById('overlay-img');
                        img.src = src;
                        overlay.style.display = 'flex';
                    }
                    function closeImage() {
                        document.getElementById('overlay').style.display = 'none';
                    }
                </script>
            </body>
        </html>
    "#);

    Ok(Html(html))
}







async fn portrait_photos_invite(
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {

    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT filename, description, category FROM photos WHERE category = 'portrait'"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
        <html>
            <head>
                <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <style>
                    body {
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                        background-color: #d8d8d0;
                        text-align: center;
                        margin: 0;
                        padding: 0;
                    }
                    h1 {
                        color: #222;
                        margin: 30px 0;
                        font-size: 2em;
                    }
                    .gallery {
                        display: grid;
                        grid-template-columns: minmax(0, 1fr);
                        gap: 22px;
                        padding: 20px;
                        max-width: 780px;
                        margin: 0 auto;
                    }
                    .photo-card {
                        background: #dcddd5;
                        border: 1px solid #d1d1c8;
                        box-shadow: 0 6px 18px rgba(0,0,0,0.035);
                        overflow: hidden;
                        width: 100%;
                        max-width: none;
                        transition: transform 0.3s, box-shadow 0.3s;
                    }
                    .photo-card:hover {
                        transform: translateY(-5px);
                        box-shadow: 0 10px 24px rgba(0,0,0,0.07);
                    }
                    .photo-card img {
                        width: 100%;
                        height: auto;
                        display: block;
                        border-radius: 0;
                        cursor: pointer;
                        transition: transform 0.3s ease;
                    }
                    .photo-card .desc {
                        padding: 15px;
                        text-align: left;
                    }
                    .photo-card .desc p {
                        margin: 5px 0;
                        color: #666;
                    }
                    .photo-card .desc span {
                        font-weight: bold;
                        color: #333;
                    }
                    .btn {
                        margin: 5px;
                        background-color: #d8d8d0;
                        border: none;
                        color: #222;
                        padding: 0.8em 1.5em;
                        font-size: 1em;
                        cursor: pointer;
                        text-decoration: none;
                        border-radius: 4px;
                        display: inline-block;
                        transition: background-color 0.3s;
                    }
                    .btn:hover {
                        background-color: #c6c6c0;
                    }
                    .actions {
                        margin: 20px 0;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 10px;
                    }
                    .filters {
                        display: flex;
                        justify-content: center;
                        flex-wrap: wrap;
                        gap: 10px;
                    }
                    @media (max-width: 600px) {
                        .btn {
                            width: 80%;
                        }
                        .photo-card {
                            width: 95%;
                        }
                    }
                </style>

                <script>
                    document.addEventListener("DOMContentLoaded", function() {
                        const images = document.querySelectorAll(".photo-card img");
                        images.forEach(img => {
                            img.addEventListener("click", () => {
                                const overlay = document.createElement("div");
                                overlay.style.position = "fixed";
                                overlay.style.top = 0;
                                overlay.style.left = 0;
                                overlay.style.width = "100%";
                                overlay.style.height = "100%";
                                overlay.style.backgroundColor = "rgba(0,0,0,0.9)";
                                overlay.style.display = "flex";
                                overlay.style.alignItems = "center";
                                overlay.style.justifyContent = "center";
                                overlay.style.zIndex = "1000";

                                const bigImg = document.createElement("img");
                                bigImg.src = img.src;
                                bigImg.style.maxWidth = "95%";
                                bigImg.style.maxHeight = "95%";
                                bigImg.style.borderRadius = "10px";
                                bigImg.style.boxShadow = "0 0 20px rgba(0, 0, 0, 1)";
                                overlay.appendChild(bigImg);

                                overlay.addEventListener("click", () => overlay.remove());
                                document.body.appendChild(overlay);
                            });
                        });
                    });
                </script>
            </head>
            <body>
                <h1>Galerie - Portrait</h1>

                <div class='actions'>
                    <a class='btn' href='/'>Accueil</a>
                    <div class='filters'>
                        <a class='btn' href='/photo_invite'>Tout</a>
                        <a class='btn' href='/photo_invite/animaux'>Animaux</a>
                        <a class='btn' href='/photo_invite/paysage'>Paysage</a>
                    </div>
                </div>

                <div class='gallery'>
    "#);

    for photo in rows {
        let filename = html_escape(&photo.filename);
        let category = html_escape(&photo.category);
        let description = html_escape(&photo.description);
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
            filename,
            category,
            description
        ));
    }

    html.push_str("</div></body></html>");

    Ok(Html(html))
}


async fn animaux_photos_invite(
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {

    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT filename, description, category FROM photos WHERE category = 'animaux'"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
        <html>
            <head>
                <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <style>
                    body {
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                        background-color: #d8d8d0;
                        text-align: center;
                        margin: 0;
                        padding: 0;
                    }
                    h1 {
                        color: #222;
                        margin: 30px 0;
                        font-size: 2em;
                    }
                    .gallery {
                        display: grid;
                        grid-template-columns: minmax(0, 1fr);
                        gap: 22px;
                        padding: 20px;
                        max-width: 780px;
                        margin: 0 auto;
                    }
                    .photo-card {
                        background: #dcddd5;
                        border: 1px solid #d1d1c8;
                        box-shadow: 0 6px 18px rgba(0,0,0,0.035);
                        overflow: hidden;
                        width: 100%;
                        max-width: none;
                        transition: transform 0.3s, box-shadow 0.3s;
                        cursor: pointer; /* 🔹 clic actif */
                    }
                    .photo-card:hover {
                        transform: translateY(-5px);
                        box-shadow: 0 10px 24px rgba(0,0,0,0.07);
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
                        color: #666;
                    }
                    .photo-card .desc span {
                        font-weight: bold;
                        color: #333;
                    }
                    .btn {
                        margin: 5px;
                        background-color: #d8d8d0;
                        border: none;
                        color: #222;
                        padding: 0.8em 1.5em;
                        font-size: 1em;
                        cursor: pointer;
                        text-decoration: none;
                        border-radius: 4px;
                        display: inline-block;
                        transition: background-color 0.3s;
                    }
                    .btn:hover {
                        background-color: #c6c6c0;
                    }
                    .actions {
                        margin: 20px 0;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 10px;
                    }
                    .filters {
                        display: flex;
                        justify-content: center;
                        flex-wrap: wrap;
                        gap: 10px;
                    }
                    @media (max-width: 600px) {
                        .btn {
                            width: 80%;
                        }
                        .photo-card {
                            width: 95%;
                        }
                    }

                    /* 🔹 Overlay pour zoom image */
                    .overlay {
                        display: none;
                        position: fixed;
                        top: 0;
                        left: 0;
                        width: 100%;
                        height: 100%;
                        background: rgba(0, 0, 0, 0.85);
                        justify-content: center;
                        align-items: center;
                        z-index: 1000;
                    }
                    .overlay img {
                        max-width: 95%;
                        max-height: 90%;
                        border-radius: 10px;
                        box-shadow: 0 0 20px rgba(0, 0, 0, 1);
                    }
                </style>
            </head>
            <body>
                <h1>Galerie - Animaux</h1>

                <div class='actions'>
                    <a class='btn' href='/'>Accueil</a>
                    <div class='filters'>
                        <a class='btn' href='/photo_invite'>Tout</a>
                        <a class='btn' href='/photo_invite/portrait'>Portrait</a>
                        <a class='btn' href='/photo_invite/paysage'>Paysage</a>
                    </div>
                </div>

                <div class='gallery'>
    "#);

    for photo in rows {
        let filename = html_escape(&photo.filename);
        let category = html_escape(&photo.category);
        let description = html_escape(&photo.description);
        html.push_str(&format!(
            r#"
                <div class='photo-card' onclick="openImage('/images/{0}')">
                    <img src='/images/{0}' alt='{1}'/>
                    <div class='desc'>
                        <p><span>Catégorie:</span> {1}</p>
                        <p><span>Description:</span> {2}</p>
                    </div>
                </div>
            "#,
            filename,
            category,
            description
        ));
    }

    html.push_str(r#"
                </div>

                <!-- 🔹 Fenêtre d’image agrandie -->
                <div class="overlay" id="overlay" onclick="closeImage()">
                    <img id="overlay-img" src="" alt=""/>
                </div>

                <script>
                    function openImage(src) {
                        const overlay = document.getElementById('overlay');
                        const img = document.getElementById('overlay-img');
                        img.src = src;
                        overlay.style.display = 'flex';
                    }
                    function closeImage() {
                        document.getElementById('overlay').style.display = 'none';
                    }
                </script>
            </body>
        </html>
    "#);

    Ok(Html(html))
}


async fn paysage_photos_invite(
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {

    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT filename, description, category FROM photos WHERE category = 'paysage'"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
        <html>
            <head>
                <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <style>
                    body {
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                        background-color: #d8d8d0;
                        text-align: center;
                        margin: 0;
                        padding: 0;
                    }
                    h1 {
                        color: #222;
                        margin: 30px 0;
                        font-size: 2em;
                    }
                    .gallery {
                        display: grid;
                        grid-template-columns: minmax(0, 1fr);
                        gap: 22px;
                        padding: 20px;
                        max-width: 780px;
                        margin: 0 auto;
                    }
                    .photo-card {
                        background: #dcddd5;
                        border: 1px solid #d1d1c8;
                        box-shadow: 0 6px 18px rgba(0,0,0,0.035);
                        overflow: hidden;
                        width: 100%; /* 🔹 width responsive */
                        max-width: none; /* 🔹 limite sur desktop */
                        transition: transform 0.3s, box-shadow 0.3s;
                    }
                    .photo-card:hover {
                        transform: translateY(-5px);
                        box-shadow: 0 10px 24px rgba(0,0,0,0.07);
                    }
                    .photo-card img {
                        width: 100%;
                        height: auto;
                        display: block;
                        border-radius: 0;
                        cursor: pointer;
                        transition: transform 0.3s ease;
                    }
                    .photo-card .desc {
                        padding: 15px;
                        text-align: left;
                    }
                    .photo-card .desc p {
                        margin: 5px 0;
                        color: #666;
                    }
                    .photo-card .desc span {
                        font-weight: bold;
                        color: #333;
                    }
                    .btn {
                        margin: 5px;
                        background-color: #d8d8d3;
                        border: none;
                        color: #222;
                        padding: 0.8em 1.5em;
                        font-size: 1em;
                        cursor: pointer;
                        text-decoration: none;
                        border-radius: 4px;
                        display: inline-block;
                        transition: background-color 0.3s;
                    }
                    .btn:hover {
                        background-color: #c6c6c0;
                    }
                    .actions {
                        margin: 20px 0;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 10px;
                    }
                    .filters {
                        display: flex;
                        justify-content: center;
                        flex-wrap: wrap;
                        gap: 10px;
                    }
                    @media (max-width: 600px) {
                        .btn {
                            width: 80%;
                        }
                        .photo-card {
                            width: 95%;
                        }
                    }
                </style>

                <script>
                    document.addEventListener("DOMContentLoaded", function() {
                        const images = document.querySelectorAll(".photo-card img");
                        images.forEach(img => {
                            img.addEventListener("click", () => {
                                const overlay = document.createElement("div");
                                overlay.style.position = "fixed";
                                overlay.style.top = 0;
                                overlay.style.left = 0;
                                overlay.style.width = "100%";
                                overlay.style.height = "100%";
                                overlay.style.backgroundColor = "rgba(0,0,0,0.9)";
                                overlay.style.display = "flex";
                                overlay.style.alignItems = "center";
                                overlay.style.justifyContent = "center";
                                overlay.style.zIndex = "1000";

                                const bigImg = document.createElement("img");
                                bigImg.src = img.src;
                                bigImg.style.maxWidth = "95%";
                                bigImg.style.maxHeight = "95%";
                                bigImg.style.borderRadius = "10px";
                                bigImg.style.boxShadow = "0 0 20px rgba(0, 0, 0, 1)";
                                overlay.appendChild(bigImg);

                                overlay.addEventListener("click", () => overlay.remove());
                                document.body.appendChild(overlay);
                            });
                        });
                    });
                </script>
            </head>
            <body>
                <h1>Galerie - Paysage</h1>

                <div class='actions'>
                    <a class='btn' href='/'>Accueil</a>
                    <div class='filters'>
                        <a class='btn' href='/photo_invite'>Tout</a>
                        <a class='btn' href='/photo_invite/animaux'>Animaux</a>
                        <a class='btn' href='/photo_invite/portrait'>Portrait</a>
                    </div>
                </div>

                <div class='gallery'>
    "#);

    for photo in rows {
        let filename = html_escape(&photo.filename);
        let category = html_escape(&photo.category);
        let description = html_escape(&photo.description);
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
            filename,
            category,
            description
        ));
    }

    html.push_str("</div></body></html>");

    Ok(Html(html))
}



async fn get_photos_admin(
    cookies: Cookies,
    State(db): State<SqlitePool>,
) -> Result<Html<String>, axum::http::StatusCode> {
    if !is_admin(&cookies) {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }

    let rows = sqlx::query_as::<_, Photo>(
        r#"SELECT filename, description, category FROM photos"#
    )
    .fetch_all(&db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(r#"
    <html>
        <head>
            <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
            <style>
                body {
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    background-color: #f4f4f2;
                    color: #222;
                    margin: 0;
                    padding: 32px 20px;
                    text-align: center;
                }
                button {
                    background-color: #d8d8d3;
                    border: none;
                    color: #222;
                    padding: 0.8em 1.2em;
                    border-radius: 4px;
                    cursor: pointer;
                }
                button:hover {
                    background-color: #c6c6c0;
                }
                .admin-gallery {
                    display: grid;
                    grid-template-columns: minmax(0, 1fr);
                    gap: 22px;
                    max-width: 780px;
                    margin: 24px auto;
                }
                .admin-card {
                    background: #dcddd5;
                    border: 1px solid #d1d1c8;
                    border-radius: 6px;
                    padding: 12px;
                    box-shadow: 0 6px 18px rgba(0,0,0,0.035);
                    text-align: left;
                }
                .admin-card img {
                    width: 100%;
                    height: auto;
                    display: block;
                    border-radius: 4px;
                }
                .admin-card p {
                    color: #666;
                    line-height: 1.5;
                }
            </style>
        </head>
        <body>
            <h1>Photos</h1>
            <form action="/homepage_admin">
                <button>Accueil</button>
            </form>
            <div class="admin-gallery">
    "#);

    for photo in rows {
        let filename = html_escape(&photo.filename);
        let description = html_escape(&photo.description);
        html.push_str(&format!(
            r#"
            <div class="admin-card">
                <img src="/images/{0}" /><br/>
                <p>{1}</p>

                <form action="/delete" method="post">
                    <input type="hidden" name="filename" value="{0}" />
                    <button type="submit">Supprimer</button>
                </form>
            </div>
            "#,
            filename, description
        ));
    }
    html.push_str("</div></body></html>");

    Ok(Html(html))
}



async fn upload_photo(
    cookies: Cookies,
    State(db): State<SqlitePool>,
    mut multipart: Multipart,
) -> Result<Redirect, String> {

    if !is_admin(&cookies) {
        return Err("Accès refusé".to_string());
    }

    let mut filename = String::new();
    let mut description = String::new();
    let mut category = "autre".to_string(); // valeur par défaut

    while let Some(field) = multipart.next_field().await.map_err(|e| e.to_string())? {
        match field.name() {
            Some("file") => {
                filename = safe_filename(field.file_name().unwrap_or("file.jpg"))
                    .ok_or_else(|| "Nom de fichier invalide".to_string())?;
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

    Ok(Redirect::to("/homepage_admin"))
}




async fn supp_photo(
    cookies: Cookies,
    State(db): State<SqlitePool>,
    Form(payload): Form<DeletePhoto>,
) -> Result<Redirect, String> {

    if !is_admin(&cookies) {
        return Err("Accès refusé".to_string());
    }
    let filename = safe_filename(&payload.filename)
        .ok_or_else(|| "Nom de fichier invalide".to_string())?;
    sqlx::query("DELETE FROM photos WHERE filename = ?")
        .bind(&filename)
        .execute(&db)
        .await
        .map_err(|e| e.to_string())?;
    let filepath = format!("images/{}", filename);
    if tokio::fs::remove_file(&filepath).await.is_err() {
        return Err("Erreur lors de la suppression du fichier".to_string());
    }
    Ok(Redirect::to("/homepage_admin"))
}  
