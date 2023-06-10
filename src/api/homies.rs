use axum::Json;

#[derive(serde::Deserialize, Debug)]
pub struct CreateHomieRequest {
    pub name: String,
}

impl CreateHomieRequest {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[tracing::instrument(name = "Creating a new homie")]
pub async fn create_homie(Json(request): Json<CreateHomieRequest>) {
    println!("Creating homie: {}", request.name);
}
