use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: PathBuf) -> Self {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create database directory");
        }
        let conn = Connection::open(&path).unwrap_or_else(|e| {
            panic!("Failed to open database at {}: {}", path.display(), e)
        });
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                question_id INTEGER DEFAULT 0,
                image_path TEXT DEFAULT '',
                student_answer TEXT DEFAULT '',
                ai_score REAL DEFAULT 0,
                teacher_score REAL DEFAULT 0,
                confidence REAL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                question_title TEXT DEFAULT '',
                max_score REAL DEFAULT 100,
                reference_answer TEXT DEFAULT '',
                rubric TEXT DEFAULT '',
                deductions TEXT DEFAULT '',
                comment TEXT DEFAULT '',
                result_json TEXT DEFAULT '',
                image_data TEXT DEFAULT '',
                reference_answer_image TEXT DEFAULT ''
            );
            CREATE TABLE IF NOT EXISTS exam (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL DEFAULT '',
                course TEXT NOT NULL DEFAULT '',
                questions_json TEXT DEFAULT '',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS rubric (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                question_title TEXT NOT NULL DEFAULT '',
                max_score REAL NOT NULL DEFAULT 100,
                reference_answer TEXT NOT NULL DEFAULT '',
                rubric TEXT NOT NULL DEFAULT '',
                sort_order INTEGER NOT NULL DEFAULT 0,
                reference_answer_image TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                level TEXT NOT NULL DEFAULT 'INFO',
                message TEXT NOT NULL DEFAULT '',
                source TEXT NOT NULL DEFAULT '',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );"
        ).expect("Failed to create tables");
        Database { conn }
    }

    pub fn get_conn(&self) -> &Connection {
        &self.conn
    }

    // History methods
    pub fn list_history(&self) -> Vec<serde_json::Value> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM history ORDER BY created_at DESC LIMIT 100"
        ).unwrap();
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let question_id: i64 = row.get(1)?;
            let image_path: String = row.get(2)?;
            let student_answer: String = row.get(3)?;
            let ai_score: f64 = row.get(4)?;
            let teacher_score: f64 = row.get(5)?;
            let confidence: f64 = row.get(6)?;
            let created_at: String = row.get(7)?;
            let question_title: String = row.get(8)?;
            let max_score: f64 = row.get(9)?;
            let reference_answer: String = row.get(10)?;
            let rubric: String = row.get(11)?;
            let deductions: String = row.get(12)?;
            let comment: String = row.get(13)?;
            let result_json: String = row.get(14)?;
            let image_data: String = row.get(15)?;
            let reference_answer_image: String = row.get(16)?;
            Ok(serde_json::json!({
                "id": id, "question_id": question_id, "image_path": image_path,
                "student_answer": student_answer, "ai_score": ai_score,
                "teacher_score": teacher_score, "confidence": confidence,
                "created_at": created_at, "question_title": question_title,
                "max_score": max_score, "reference_answer": reference_answer,
                "rubric": rubric, "deductions": deductions, "comment": comment,
                "result_json": result_json, "image_data": image_data,
                "reference_answer_image": reference_answer_image
            }))
        }).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn get_history_by_id(&self, id: i64) -> Option<serde_json::Value> {
        self.conn.prepare(
            "SELECT * FROM history WHERE id = ?"
        ).ok().and_then(|mut stmt| {
            stmt.query_row(params![id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "question_id": row.get::<_, i64>(1)?,
                    "image_path": row.get::<_, String>(2)?,
                    "student_answer": row.get::<_, String>(3)?,
                    "ai_score": row.get::<_, f64>(4)?,
                    "teacher_score": row.get::<_, f64>(5)?,
                    "confidence": row.get::<_, f64>(6)?,
                    "created_at": row.get::<_, String>(7)?,
                    "question_title": row.get::<_, String>(8)?,
                    "max_score": row.get::<_, f64>(9)?,
                    "reference_answer": row.get::<_, String>(10)?,
                    "rubric": row.get::<_, String>(11)?,
                    "deductions": row.get::<_, String>(12)?,
                    "comment": row.get::<_, String>(13)?,
                    "result_json": row.get::<_, String>(14)?,
                    "image_data": row.get::<_, String>(15)?,
                    "reference_answer_image": row.get::<_, String>(16)?
                }))
            }).ok()
        })
    }

    pub fn delete_history(&self, id: i64) -> bool {
        self.conn.execute("DELETE FROM history WHERE id = ?", params![id]).is_ok()
    }

    pub fn insert_history(&self, data: &serde_json::Value) -> bool {
        self.conn.execute(
            "INSERT INTO history (question_id, image_path, student_answer, ai_score, teacher_score, confidence, question_title, max_score, reference_answer, rubric, deductions, comment, result_json, image_data, reference_answer_image) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                data["questionId"].as_i64().unwrap_or(0),
                data["imagePath"].as_str().unwrap_or(""),
                data["studentAnswer"].as_str().unwrap_or(""),
                data["aiScore"].as_f64().unwrap_or(0.0),
                data["teacherScore"].as_f64().unwrap_or(0.0),
                data["confidence"].as_f64().unwrap_or(0.0),
                data["questionTitle"].as_str().unwrap_or(""),
                data["maxScore"].as_f64().unwrap_or(100.0),
                data["referenceAnswer"].as_str().unwrap_or(""),
                data["rubric"].as_str().unwrap_or(""),
                data["deductions"].as_str().unwrap_or("[]"),
                data["comment"].as_str().unwrap_or(""),
                data["resultJson"].as_str().unwrap_or(""),
                data["imageData"].as_str().unwrap_or(""),
                data["referenceAnswerImage"].as_str().unwrap_or(""),
            ]
        ).is_ok()
    }

    // Rubric methods
    pub fn get_rubric_questions(&self) -> Vec<serde_json::Value> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM rubric ORDER BY sort_order"
        ).unwrap();
        let rows = stmt.query_map([], |row| {
            let q_title: String = row.get(1)?;
            let q_max: f64 = row.get(2)?;
            let q_ref: String = row.get(3)?;
            let q_rubric: String = row.get(4)?;
            let q_img: String = row.get(6)?;
            let ref_img = if q_img.is_empty() || q_img.starts_with("data:") { q_img } else { format!("data:image/png;base64,{}", q_img) };
            Ok(serde_json::json!({
                "questionTitle": q_title,
                "maxScore": q_max,
                "referenceAnswer": q_ref,
                "rubric": q_rubric,
                "referenceAnswerImage": ref_img
            }))
        }).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn clear_rubric(&self) {
        let _ = self.conn.execute("DELETE FROM rubric", []);
    }

    pub fn insert_rubric_question(&self, q: &serde_json::Value, sort_order: i32) -> bool {
        self.conn.execute(
            "INSERT INTO rubric (question_title, max_score, reference_answer, rubric, sort_order, reference_answer_image) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                q["questionTitle"].as_str().unwrap_or(""),
                q["maxScore"].as_f64().unwrap_or(100.0),
                q["referenceAnswer"].as_str().unwrap_or(""),
                q["rubric"].as_str().unwrap_or(""),
                sort_order,
                q["referenceAnswerImage"].as_str().unwrap_or(""),
            ]
        ).is_ok()
    }

    // Template (exam) methods
    pub fn list_templates(&self) -> Vec<serde_json::Value> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM exam ORDER BY created_at DESC"
        ).unwrap();
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let course: String = row.get(2)?;
            let questions_json: String = row.get(3)?;
            let created_at: String = row.get(4)?;
            let questions: serde_json::Value = serde_json::from_str(&questions_json).unwrap_or(serde_json::Value::Array(vec![]));
            Ok(serde_json::json!({
                "id": id, "name": name, "course": course,
                "questions": questions, "created_at": created_at
            }))
        }).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn get_template_by_id(&self, id: i64) -> Option<serde_json::Value> {
        self.conn.prepare("SELECT * FROM exam WHERE id = ?").ok().and_then(|mut stmt| {
            stmt.query_row(params![id], |row| {
                let questions_json: String = row.get(3)?;
                let questions: serde_json::Value = serde_json::from_str(&questions_json).unwrap_or(serde_json::Value::Array(vec![]));
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "course": row.get::<_, String>(2)?,
                    "questions": questions,
                    "created_at": row.get::<_, String>(4)?
                }))
            }).ok()
        })
    }

    pub fn save_template(&self, name: &str, course: &str, questions: &str) -> i64 {
        self.conn.execute(
            "INSERT INTO exam (name, course, questions_json) VALUES (?1, ?2, ?3)",
            params![name, course, questions]
        ).ok();
        self.conn.last_insert_rowid()
    }

    pub fn delete_template(&self, id: i64) -> bool {
        self.conn.execute("DELETE FROM exam WHERE id = ?", params![id]).is_ok()
    }

    // Log methods
    pub fn write_log(&self, level: &str, message: &str, source: &str) {
        let _ = self.conn.execute(
            "INSERT INTO logs (level, message, source) VALUES (?1, ?2, ?3)",
            params![level, message, source],
        );
    }

    pub fn list_logs(&self, limit: i64) -> Vec<serde_json::Value> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM logs ORDER BY id DESC LIMIT ?"
        ).unwrap();
        let rows = stmt.query_map(params![limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "level": row.get::<_, String>(1)?,
                "message": row.get::<_, String>(2)?,
                "source": row.get::<_, String>(3)?,
                "createdAt": row.get::<_, String>(4)?
            }))
        }).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn clear_logs(&self) {
        let _ = self.conn.execute("DELETE FROM logs", []);
    }
}
