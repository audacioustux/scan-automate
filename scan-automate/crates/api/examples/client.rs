use api::serve;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(html));
    serve(app, 3000).await;
}

async fn html() -> impl IntoResponse {
    Html(
        r#"
        <!-- a input field to enter the url -->
        <input type="text" id="url" name="url" placeholder="Enter the url" />
        <!-- a input field to enter the email -->
        <input type="text" id="email" name="email" placeholder="Enter the email" />
        <!-- a button to submit the form -->
        <button id="submit" onclick="submit()">Submit</button>
        <!-- print the response -->
        <div id="response"></div>

        <script>
            // get the url and email from the input fields
            const url = document.getElementById("url");
            const email = document.getElementById("email");
            // get the response div
            const response = document.getElementById("response");

            // submit the form
            async function submit() {
                // send a post request to the api
                const res = await fetch("http://localhost:3000/scans", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        email: email.value,
                        zap: {
                            uri: url.value,
                        },
                        rustscan: {
                            uri: url.value,
                        },
                    }),
                });
                // get the response from the api
                const data = await res.json();
                // print the response
                response.innerHTML = JSON.stringify(data);
            }
        </script>

        <!-- a input field to enter scan id -->
        <input type="text" id="scan-id" name="scan-id" placeholder="Enter the scan id" />
        <!-- a button to submit the form -->
        <button id="submit" onclick="progress()">Progress</button>
        <!-- print the response -->
        <div id="progress"></div>

        <script>
            // get the scan id from the input field
            const scanId = document.getElementById("scan-id");
            // get the progress div
            const progress = document.getElementById("progress");

            // get the progress of the scan
            async function progress() {
                // send a get request to the api
                const res = await fetch(`http://localhost:3000/scans/progress/${scanId.value}`);
                // get the response from the api
                const data = await res.json();
                // print the response
                progress.innerHTML = JSON.stringify(data);
            }
        </script>
        "#,
    )
}
