use anyhow::Result;
use kcl_lib::{
    lint::{checks, Discovered},
    ExecutorContext, UnitLength,
};
use pyo3::{
    prelude::PyModuleMethods, pyclass, pyfunction, pymethods, pymodule, types::PyModule, wrap_pyfunction, Bound, PyErr,
    PyResult,
};
use serde::{Deserialize, Serialize};

fn tokio() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

/// The variety of image formats snapshots may be exported to.
#[derive(Serialize, Deserialize, PartialEq, Hash, Debug, Clone, Copy)]
#[pyclass(eq, eq_int)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    /// .png format
    Png,
    /// .jpeg format
    Jpeg,
}

impl From<ImageFormat> for kittycad_modeling_cmds::ImageFormat {
    fn from(format: ImageFormat) -> Self {
        match format {
            ImageFormat::Png => kittycad_modeling_cmds::ImageFormat::Png,
            ImageFormat::Jpeg => kittycad_modeling_cmds::ImageFormat::Jpeg,
        }
    }
}

/// A file that was exported from the engine.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[pyclass]
pub struct ExportFile {
    /// Binary contents of the file.
    pub contents: Vec<u8>,
    /// Name of the file.
    pub name: String,
}

impl From<kittycad_modeling_cmds::shared::ExportFile> for ExportFile {
    fn from(file: kittycad_modeling_cmds::shared::ExportFile) -> Self {
        ExportFile {
            contents: file.contents.0,
            name: file.name,
        }
    }
}

impl From<kittycad_modeling_cmds::websocket::RawFile> for ExportFile {
    fn from(file: kittycad_modeling_cmds::websocket::RawFile) -> Self {
        ExportFile {
            contents: file.contents,
            name: file.name,
        }
    }
}

#[pymethods]
impl ExportFile {
    #[getter]
    fn contents(&self) -> Vec<u8> {
        self.contents.clone()
    }

    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }
}

/// The valid types of output file formats.
#[derive(Serialize, Deserialize, PartialEq, Hash, Debug, Clone)]
#[pyclass(eq, eq_int)]
#[serde(rename_all = "lowercase")]
pub enum FileExportFormat {
    /// Autodesk Filmbox (FBX) format. <https://en.wikipedia.org/wiki/FBX>
    Fbx,
    /// Binary glTF 2.0.
    ///
    /// This is a single binary with .glb extension.
    ///
    /// This is better if you want a compressed format as opposed to the human readable glTF that lacks
    /// compression.
    Glb,
    /// glTF 2.0. Embedded glTF 2.0 (pretty printed).
    ///
    /// Single JSON file with .gltf extension binary data encoded as base64 data URIs.
    ///
    /// The JSON contents are pretty printed.
    ///
    /// It is human readable, single file, and you can view the diff easily in a
    /// git commit.
    Gltf,
    /// The OBJ file format. <https://en.wikipedia.org/wiki/Wavefront_.obj_file> It may or
    /// may not have an an attached material (mtl // mtllib) within the file, but we
    /// interact with it as if it does not.
    Obj,
    /// The PLY file format. <https://en.wikipedia.org/wiki/PLY_(file_format)>
    Ply,
    /// The STEP file format. <https://en.wikipedia.org/wiki/ISO_10303-21>
    Step,
    /// The STL file format. <https://en.wikipedia.org/wiki/STL_(file_format)>
    Stl,
}

fn get_output_format(
    format: &FileExportFormat,
    src_unit: kittycad_modeling_cmds::units::UnitLength,
) -> kittycad_modeling_cmds::format::OutputFormat {
    // Zoo co-ordinate system.
    //
    // * Forward: -Y
    // * Up: +Z
    // * Handedness: Right
    let coords = kittycad_modeling_cmds::coord::System {
        forward: kittycad_modeling_cmds::coord::AxisDirectionPair {
            axis: kittycad_modeling_cmds::coord::Axis::Y,
            direction: kittycad_modeling_cmds::coord::Direction::Negative,
        },
        up: kittycad_modeling_cmds::coord::AxisDirectionPair {
            axis: kittycad_modeling_cmds::coord::Axis::Z,
            direction: kittycad_modeling_cmds::coord::Direction::Positive,
        },
    };

    match format {
        FileExportFormat::Fbx => {
            kittycad_modeling_cmds::format::OutputFormat::Fbx(kittycad_modeling_cmds::format::fbx::export::Options {
                storage: kittycad_modeling_cmds::format::fbx::export::Storage::Binary,
                created: None,
            })
        }
        FileExportFormat::Glb => {
            kittycad_modeling_cmds::format::OutputFormat::Gltf(kittycad_modeling_cmds::format::gltf::export::Options {
                storage: kittycad_modeling_cmds::format::gltf::export::Storage::Binary,
                presentation: kittycad_modeling_cmds::format::gltf::export::Presentation::Compact,
            })
        }
        FileExportFormat::Gltf => {
            kittycad_modeling_cmds::format::OutputFormat::Gltf(kittycad_modeling_cmds::format::gltf::export::Options {
                storage: kittycad_modeling_cmds::format::gltf::export::Storage::Embedded,
                presentation: kittycad_modeling_cmds::format::gltf::export::Presentation::Pretty,
            })
        }
        FileExportFormat::Obj => {
            kittycad_modeling_cmds::format::OutputFormat::Obj(kittycad_modeling_cmds::format::obj::export::Options {
                coords,
                units: src_unit,
            })
        }
        FileExportFormat::Ply => {
            kittycad_modeling_cmds::format::OutputFormat::Ply(kittycad_modeling_cmds::format::ply::export::Options {
                storage: kittycad_modeling_cmds::format::ply::export::Storage::Ascii,
                coords,
                selection: kittycad_modeling_cmds::format::Selection::DefaultScene,
                units: src_unit,
            })
        }
        FileExportFormat::Step => {
            kittycad_modeling_cmds::format::OutputFormat::Step(kittycad_modeling_cmds::format::step::export::Options {
                coords,
                created: None,
            })
        }
        FileExportFormat::Stl => {
            kittycad_modeling_cmds::format::OutputFormat::Stl(kittycad_modeling_cmds::format::stl::export::Options {
                storage: kittycad_modeling_cmds::format::stl::export::Storage::Ascii,
                coords,
                units: src_unit,
                selection: kittycad_modeling_cmds::format::Selection::DefaultScene,
            })
        }
    }
}

async fn new_context(units: UnitLength) -> Result<ExecutorContext> {
    let ctx = ExecutorContext::new_with_default_client(units).await?;
    Ok(ctx)
}

/// Execute the kcl code.
#[pyfunction]
async fn execute(code: String, units: UnitLength) -> PyResult<()> {
    tokio()
        .spawn(async move {
            let program = kcl_lib::Program::parse(&code).map_err(PyErr::from)?;
            let ctx = new_context(units)
                .await
                .map_err(|err| pyo3::exceptions::PyException::new_err(err.to_string()))?;
            // Execute the program.
            ctx.run(&program, &mut Default::default()).await?;

            Ok(())
        })
        .await
        .map_err(|err| pyo3::exceptions::PyException::new_err(err.to_string()))?
}

/// Execute the kcl code and snapshot it in a specific format.
#[pyfunction]
async fn execute_and_snapshot(code: String, units: UnitLength, image_format: ImageFormat) -> PyResult<Vec<u8>> {
    tokio()
        .spawn(async move {
            let program = kcl_lib::Program::parse(&code).map_err(PyErr::from)?;
            let ctx = new_context(units)
                .await
                .map_err(|err| pyo3::exceptions::PyException::new_err(err.to_string()))?;
            // Execute the program.
            ctx.run(&program, &mut Default::default()).await?;

            // Zoom to fit.
            ctx.engine
                .send_modeling_cmd(
                    uuid::Uuid::new_v4(),
                    kcl_lib::SourceRange::default(),
                    kittycad_modeling_cmds::ModelingCmd::ZoomToFit(kittycad_modeling_cmds::ZoomToFit {
                        object_ids: Default::default(),
                        padding: 0.1,
                        animated: false,
                    }),
                )
                .await?;

            // Send a snapshot request to the engine.
            let resp = ctx
                .engine
                .send_modeling_cmd(
                    uuid::Uuid::new_v4(),
                    kcl_lib::SourceRange::default(),
                    kittycad_modeling_cmds::ModelingCmd::TakeSnapshot(kittycad_modeling_cmds::TakeSnapshot {
                        format: image_format.into(),
                    }),
                )
                .await?;

            let kittycad_modeling_cmds::websocket::OkWebSocketResponseData::Modeling {
                modeling_response: kittycad_modeling_cmds::ok_response::OkModelingCmdResponse::TakeSnapshot(data),
            } = resp
            else {
                return Err(pyo3::exceptions::PyException::new_err(format!(
                    "Unexpected response from engine: {:?}",
                    resp
                )));
            };

            Ok(data.contents.0)
        })
        .await
        .map_err(|err| pyo3::exceptions::PyException::new_err(err.to_string()))?
}

/// Execute the kcl code and export it to a specific file format.
#[pyfunction]
async fn execute_and_export(
    code: String,
    units: UnitLength,
    export_format: FileExportFormat,
) -> PyResult<Vec<ExportFile>> {
    tokio()
        .spawn(async move {
            let program = kcl_lib::Program::parse(&code).map_err(PyErr::from)?;
            let ctx = new_context(units)
                .await
                .map_err(|err| pyo3::exceptions::PyException::new_err(err.to_string()))?;
            // Execute the program.
            ctx.run(&program, &mut Default::default()).await?;

            // This will not return until there are files.
            let resp = ctx
                .engine
                .send_modeling_cmd(
                    uuid::Uuid::new_v4(),
                    kcl_lib::SourceRange::default(),
                    kittycad_modeling_cmds::ModelingCmd::Export(kittycad_modeling_cmds::Export {
                        entity_ids: vec![],
                        format: get_output_format(&export_format, units.into()),
                    }),
                )
                .await?;

            let kittycad_modeling_cmds::websocket::OkWebSocketResponseData::Export { files } = resp else {
                return Err(pyo3::exceptions::PyException::new_err(format!(
                    "Unexpected response from engine: {:?}",
                    resp
                )));
            };

            Ok(files.into_iter().map(ExportFile::from).collect())
        })
        .await
        .map_err(|err| pyo3::exceptions::PyException::new_err(err.to_string()))?
}

/// Format the kcl code.
#[pyfunction]
fn format(code: String) -> PyResult<String> {
    let program = kcl_lib::Program::parse(&code).map_err(PyErr::from)?;
    let recasted = program.recast();

    Ok(recasted)
}

/// Lint the kcl code.
#[pyfunction]
fn lint(code: String) -> PyResult<Vec<Discovered>> {
    let program = kcl_lib::Program::parse(&code).map_err(PyErr::from)?;
    let lints = program
        .lint(checks::lint_variables)
        .map_err(|err| pyo3::exceptions::PyException::new_err(err.to_string()))?;

    Ok(lints)
}

/// The kcl python module.
#[pymodule]
fn kcl(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add our types to the module.
    m.add_class::<ImageFormat>()?;
    m.add_class::<ExportFile>()?;
    m.add_class::<FileExportFormat>()?;
    m.add_class::<UnitLength>()?;
    m.add_class::<Discovered>()?;

    // Add our functions to the module.
    m.add_function(wrap_pyfunction!(execute, m)?)?;
    m.add_function(wrap_pyfunction!(execute_and_snapshot, m)?)?;
    m.add_function(wrap_pyfunction!(execute_and_export, m)?)?;
    m.add_function(wrap_pyfunction!(format, m)?)?;
    m.add_function(wrap_pyfunction!(lint, m)?)?;
    Ok(())
}
