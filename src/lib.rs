use anyhow::Result;
use kcl_lib::{
    ast::types::Program,
    executor::{ExecutorContext, ExecutorSettings, ProgramMemory},
    settings::types::UnitLength,
    token::Token,
};
use pyo3::pyfunction;
use pyo3::pymodule;
use pyo3::types::PyDict;
use pyo3::types::PyModule;
use pyo3::wrap_pyfunction;
use pyo3::Bound;
use pyo3::PyErr;
use pyo3::PyResult;
use pyo3::ToPyObject;
use serde::de::DeserializeOwned;
use serde::Serialize;

async fn new_context(units: UnitLength) -> Result<ExecutorContext> {
    let user_agent = concat!(env!("CARGO_PKG_NAME"), ".rs/", env!("CARGO_PKG_VERSION"),);
    let http_client = reqwest::Client::builder()
        .user_agent(user_agent)
        // For file conversions we need this to be long.
        .timeout(std::time::Duration::from_secs(600))
        .connect_timeout(std::time::Duration::from_secs(60));
    let ws_client = reqwest::Client::builder()
        .user_agent(user_agent)
        // For file conversions we need this to be long.
        .timeout(std::time::Duration::from_secs(600))
        .connect_timeout(std::time::Duration::from_secs(60))
        .connection_verbose(true)
        .tcp_keepalive(std::time::Duration::from_secs(600))
        .http1_only();

    let token = std::env::var("KITTYCAD_API_TOKEN").expect("KITTYCAD_API_TOKEN not set");

    // Create the client.
    let mut client = kittycad::Client::new_from_reqwest(token, http_client, ws_client);
    // Set a local engine address if it's set.
    if let Ok(addr) = std::env::var("KITTYCAD_HOST") {
        client.set_base_url(addr);
    }

    let ctx = ExecutorContext::new(
        &client,
        ExecutorSettings {
            units,
            highlight_edges: true,
            enable_ssao: false,
        },
    )
    .await?;
    Ok(ctx)
}

fn to_dict<T: Serialize>(value: T) -> Result<PyDict> {
    let v = serde_json::to_value(value)?;
    Ok(serde_json::from_value(v)?)
}

fn from_dict<T: DeserializeOwned>(dict: &PyDict) -> Result<T> {
    Ok(serde_json::from_value(dict.to_object())?)
}

/// Tokenize the kcl code.
#[pyfunction]
fn tokenize(code: &str) -> PyResult<Vec<Token>> {
    let tokens = kcl_lib::token::lexer(code).map_err(PyErr::from)?;
    Ok(tokens)
}

/// Parse the kcl tokens.
#[pyfunction]
fn parse(tokens: Vec<Token>) -> PyResult<PyDict> {
    let parser = kcl_lib::parser::Parser::new(tokens);
    let program = parser.ast().map_err(PyErr::from)?;
    Ok(to_dict(program).map_err(|err| pyo3::exceptions::PyException::new_err(err.to_string()))?)
}

/// Execute the kcl program.
#[pyfunction]
async fn execute(program: PyDict, units: UnitLength) -> PyResult<PyDict> {
    let program: Program =
        from_dict(&program).map_err(|err| pyo3::exceptions::PyException::new_err(err.to_string()))?;
    let ctx = new_context(units)
        .await
        .map_err(|err| pyo3::exceptions::PyException::new_err(err.to_string()))?;
    let memory = ctx.run(program, None).await.map_err(PyErr::from)?;
    Ok(to_dict(memory).map_err(|err| pyo3::exceptions::PyException::new_err(err.to_string()))?)
}

/// A Python module implemented in Rust.
#[pymodule]
fn kcl(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(tokenize, m)?)?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(execute, m)?)?;
    Ok(())
}
