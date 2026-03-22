use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OpStatus {
    Pending,
    Running,
    Done,
    Failed(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OpType {
    VideoGeneration,
    ImageGeneration,
    YoutubeUpload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Operation {
    pub id: String, // GCloud Operation ID or unique UUID
    pub op_type: OpType,
    pub status: OpStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Contexto específico
    pub prompt: String,
    pub target_file: String,      // Ruta local esperada
    pub gcs_uri: Option<String>,  // Ruta en la nube (gs://...)
    pub metadata: Option<String>, // Extra info (JSON string)
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct OperationStore {
    pub operations: Vec<Operation>,
}

impl OperationStore {
    pub fn load(path: &Path) -> io::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        if content.trim().is_empty() {
            return Ok(Self::default());
        }
        let store = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(store)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add(&mut self, op: Operation) {
        // Si ya existe, actualizamos
        if let Some(existing) = self.operations.iter_mut().find(|o| o.id == op.id) {
            *existing = op;
        } else {
            self.operations.push(op);
        }
    }

    pub fn get_pending(&self) -> Vec<&Operation> {
        self.operations
            .iter()
            .filter(|op| op.status == OpStatus::Running || op.status == OpStatus::Pending)
            .collect()
    }

    pub fn mark_done(&mut self, id: &str) {
        if let Some(op) = self.operations.iter_mut().find(|o| o.id == id) {
            op.status = OpStatus::Done;
            op.updated_at = Utc::now();
        }
    }

    pub fn mark_failed(&mut self, id: &str, reason: String) {
        if let Some(op) = self.operations.iter_mut().find(|o| o.id == id) {
            op.status = OpStatus::Failed(reason);
            op.updated_at = Utc::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_operation(id: &str, status: OpStatus) -> Operation {
        Operation {
            id: id.to_string(),
            op_type: OpType::VideoGeneration,
            status,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            prompt: "Test prompt".to_string(),
            target_file: "/tmp/test.mp4".to_string(),
            gcs_uri: Some("gs://test/video.mp4".to_string()),
            metadata: None,
        }
    }

    #[test]
    fn test_operation_store_load_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let result = OperationStore::load(temp_file.path());

        assert!(result.is_ok());
        let store = result.unwrap();
        assert_eq!(store.operations.len(), 0);
    }

    #[test]
    fn test_operation_store_load_nonexistent_file() {
        let path = std::path::Path::new("/tmp/nonexistent_test_file.json");
        let result = OperationStore::load(path);

        assert!(result.is_ok());
        let store = result.unwrap();
        assert_eq!(store.operations.len(), 0);
    }

    #[test]
    fn test_operation_store_load_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = r#"{
            "operations": [
                {
                    "id": "test-123",
                    "op_type": "VideoGeneration",
                    "status": "Done",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z",
                    "prompt": "Test",
                    "target_file": "/tmp/test.mp4",
                    "gcs_uri": null,
                    "metadata": null
                }
            ]
        }"#;

        writeln!(temp_file, "{}", test_data).unwrap();

        let result = OperationStore::load(temp_file.path());
        assert!(result.is_ok());

        let store = result.unwrap();
        assert_eq!(store.operations.len(), 1);
        assert_eq!(store.operations[0].id, "test-123");
    }

    #[test]
    fn test_operation_store_save() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut store = OperationStore::default();

        let op = create_test_operation("save-test-1", OpStatus::Done);
        store.operations.push(op);

        let result = store.save(temp_file.path());
        assert!(result.is_ok());

        // Verificar que el archivo se guardó correctamente
        let loaded_store = OperationStore::load(temp_file.path()).unwrap();
        assert_eq!(loaded_store.operations.len(), 1);
        assert_eq!(loaded_store.operations[0].id, "save-test-1");
    }

    #[test]
    fn test_operation_store_add_new_operation() {
        let mut store = OperationStore::default();
        let op = create_test_operation("add-test-1", OpStatus::Pending);

        store.add(op.clone());

        assert_eq!(store.operations.len(), 1);
        assert_eq!(store.operations[0].id, "add-test-1");
        assert_eq!(store.operations[0].status, OpStatus::Pending);
    }

    #[test]
    fn test_operation_store_add_update_existing() {
        let mut store = OperationStore::default();

        // Agregar operación inicial
        let op1 = create_test_operation("update-test-1", OpStatus::Pending);
        store.add(op1);

        assert_eq!(store.operations.len(), 1);
        assert_eq!(store.operations[0].status, OpStatus::Pending);

        // Actualizar la misma operación
        let op2 = create_test_operation("update-test-1", OpStatus::Done);
        store.add(op2);

        // Debe seguir habiendo solo 1 operación, pero actualizada
        assert_eq!(store.operations.len(), 1);
        assert_eq!(store.operations[0].status, OpStatus::Done);
    }

    #[test]
    fn test_operation_store_get_pending() {
        let mut store = OperationStore::default();

        store.add(create_test_operation("pending-1", OpStatus::Pending));
        store.add(create_test_operation("running-1", OpStatus::Running));
        store.add(create_test_operation("done-1", OpStatus::Done));
        store.add(create_test_operation(
            "failed-1",
            OpStatus::Failed("Error".to_string()),
        ));

        let pending = store.get_pending();

        // Debe retornar solo Pending y Running
        assert_eq!(pending.len(), 2);

        let ids: Vec<&str> = pending.iter().map(|op| op.id.as_str()).collect();
        assert!(ids.contains(&"pending-1"));
        assert!(ids.contains(&"running-1"));
    }

    #[test]
    fn test_operation_store_mark_done() {
        let mut store = OperationStore::default();

        let op = create_test_operation("mark-done-1", OpStatus::Running);
        store.add(op);

        assert_eq!(store.operations[0].status, OpStatus::Running);

        store.mark_done("mark-done-1");

        assert_eq!(store.operations[0].status, OpStatus::Done);
    }

    #[test]
    fn test_operation_store_mark_done_nonexistent() {
        let mut store = OperationStore::default();

        // No debe causar panic
        store.mark_done("nonexistent-id");

        assert_eq!(store.operations.len(), 0);
    }

    #[test]
    fn test_operation_store_mark_failed() {
        let mut store = OperationStore::default();

        let op = create_test_operation("mark-failed-1", OpStatus::Running);
        store.add(op);

        assert_eq!(store.operations[0].status, OpStatus::Running);

        store.mark_failed("mark-failed-1", "Connection timeout".to_string());

        match &store.operations[0].status {
            OpStatus::Failed(reason) => assert_eq!(reason, "Connection timeout"),
            _ => panic!("Expected Failed status"),
        }
    }

    #[test]
    fn test_operation_store_mark_failed_nonexistent() {
        let mut store = OperationStore::default();

        // No debe causar panic
        store.mark_failed("nonexistent-id", "Error".to_string());

        assert_eq!(store.operations.len(), 0);
    }

    #[test]
    fn test_operation_store_roundtrip() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut store = OperationStore::default();

        // Agregar varias operaciones
        store.add(create_test_operation("rt-1", OpStatus::Pending));
        store.add(create_test_operation("rt-2", OpStatus::Running));
        store.add(create_test_operation("rt-3", OpStatus::Done));

        // Guardar
        store.save(temp_file.path()).unwrap();

        // Cargar
        let loaded_store = OperationStore::load(temp_file.path()).unwrap();

        // Verificar
        assert_eq!(loaded_store.operations.len(), 3);
        assert_eq!(loaded_store.operations[0].id, "rt-1");
        assert_eq!(loaded_store.operations[1].id, "rt-2");
        assert_eq!(loaded_store.operations[2].id, "rt-3");
    }
}
