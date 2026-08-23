use rusqlite::Connection;
use std::collections::HashMap;
use std::path::Path;

use crate::models::{Category, CategoryStat, Expense, MonthStat, MonthSummary, MonthTotal, NewExpense, YearReport};

/// 内置两级分类（10 个一级大类、66 个二级小类，见 CLAUDE.md 第 5 节）
const BUILTIN_CATEGORIES: &[(&str, &[&str])] = &[
    ("餐饮食品", &["早餐", "午餐", "晚餐", "夜宵", "外卖", "买菜食材", "零食饮料", "咖啡奶茶", "聚餐宴请"]),
    ("交通出行", &["公共交通", "打车网约车", "加油充电", "停车费", "火车票", "机票", "高速过路费", "共享单车", "车辆保养维修"]),
    ("居住住房", &["房租", "房贷", "物业费", "水费", "电费", "燃气费", "宽带网费", "家具家电", "家居维修", "保洁家政"]),
    ("购物消费", &["服饰鞋包", "美妆护肤", "数码产品", "日用百货", "家用电器", "图书文具", "宠物用品", "母婴用品", "烟酒"]),
    ("娱乐休闲", &["电影演出", "游戏充值", "视频音乐会员", "运动健身", "旅游度假", "景点门票", "KTV酒吧", "兴趣爱好"]),
    ("医疗健康", &["门诊挂号", "药品购买", "住院治疗", "体检保健", "牙科眼科"]),
    ("教育学习", &["学费培训", "书籍资料", "在线课程", "考试报名"]),
    ("人情往来", &["请客送礼", "红包支出", "婚礼份子", "孝敬长辈", "慈善捐赠"]),
    ("金融保险", &["保险费用", "贷款利息", "手续费", "税费"]),
    ("其他", &["快递费用", "罚款缴费", "其他杂项"]),
];

/// 打开（或创建）数据库并完成建表与预置分类
pub fn init(db_path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path)?;
    ensure_schema(&conn)?;
    Ok(conn)
}

/// 建表与预置分类（幂等，可在任意连接上调用，供恢复后校验结构）
fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS categories (
             id        INTEGER PRIMARY KEY AUTOINCREMENT,
             name      TEXT NOT NULL,
             parent_id INTEGER REFERENCES categories(id) ON DELETE CASCADE,
             UNIQUE (parent_id, name)
         );
         CREATE TABLE IF NOT EXISTS expenses (
             id           INTEGER PRIMARY KEY AUTOINCREMENT,
             amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
             category_id  INTEGER NOT NULL REFERENCES categories(id),
             occurred_at  TEXT NOT NULL,
             note         TEXT NOT NULL DEFAULT '',
             created_at   TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
             updated_at   TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
         );
         CREATE INDEX IF NOT EXISTS idx_expenses_occurred_at ON expenses(occurred_at);",
    )?;
    seed_categories(conn)
}

/// 首次运行时写入内置分类
fn seed_categories(conn: &Connection) -> rusqlite::Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }
    let mut insert = conn.prepare("INSERT INTO categories (name, parent_id) VALUES (?1, ?2)")?;
    for (parent, children) in BUILTIN_CATEGORIES {
        insert.execute(rusqlite::params![parent, None::<i64>])?;
        let parent_id = conn.last_insert_rowid();
        for child in *children {
            insert.execute(rusqlite::params![child, parent_id])?;
        }
    }
    Ok(())
}

/// 查询全部分类（按 id 升序，大类在前、其后跟随它的二级小类）
pub fn list_categories(conn: &Connection) -> rusqlite::Result<Vec<Category>> {
    let mut stmt = conn.prepare("SELECT id, name, parent_id FROM categories ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
        })
    })?;
    rows.collect()
}

// ---------- 账单 ----------

/// 校验消费时间格式 "YYYY-MM-DD HH:MM"
/// 位置示意：YYYY(0-3) -(4) MM(5-6) -(7) DD(8-9) 空格(10) HH(11-12) :(13) MM(14-15)
fn valid_time(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 16
        && b[4] == b'-'
        && b[7] == b'-'
        && b[10] == b' '
        && b[13] == b':'
        && b.iter()
            .enumerate()
            .all(|(i, &c)| i == 4 || i == 7 || i == 10 || i == 13 || c.is_ascii_digit())
}

fn category_exists(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM categories WHERE id = ?1)",
        [id],
        |row| row.get(0),
    )
}

fn get_expense(conn: &Connection, id: i64) -> Result<Expense, String> {
    conn.query_row(
        "SELECT id, amount_cents, category_id, occurred_at, note, created_at, updated_at
         FROM expenses WHERE id = ?1",
        [id],
        |row| {
            Ok(Expense {
                id: row.get(0)?,
                amount_cents: row.get(1)?,
                category_id: row.get(2)?,
                occurred_at: row.get(3)?,
                note: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

fn validate_input(conn: &Connection, input: &NewExpense) -> Result<(), String> {
    if input.amount_cents <= 0 {
        return Err("金额必须大于 0".into());
    }
    if !valid_time(&input.occurred_at) {
        return Err("时间格式不正确，应为 YYYY-MM-DD HH:MM".into());
    }
    if !category_exists(conn, input.category_id).map_err(|e| e.to_string())? {
        return Err("分类不存在".into());
    }
    Ok(())
}

pub fn create_expense(conn: &Connection, input: &NewExpense) -> Result<Expense, String> {
    validate_input(conn, input)?;
    conn.execute(
        "INSERT INTO expenses (amount_cents, category_id, occurred_at, note)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![input.amount_cents, input.category_id, input.occurred_at, input.note],
    )
    .map_err(|e| e.to_string())?;
    get_expense(conn, conn.last_insert_rowid())
}

pub fn update_expense(conn: &Connection, id: i64, input: &NewExpense) -> Result<(), String> {
    validate_input(conn, input)?;
    let n = conn
        .execute(
            "UPDATE expenses SET amount_cents = ?1, category_id = ?2, occurred_at = ?3, note = ?4,
                    updated_at = datetime('now', 'localtime')
             WHERE id = ?5",
            rusqlite::params![
                input.amount_cents,
                input.category_id,
                input.occurred_at,
                input.note,
                id
            ],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("账单不存在".into());
    }
    Ok(())
}

pub fn delete_expense(conn: &Connection, id: i64) -> Result<(), String> {
    let n = conn
        .execute("DELETE FROM expenses WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("账单不存在".into());
    }
    Ok(())
}

/// 查询某月（"YYYY-MM"）的账单列表与合计
pub fn list_expenses(conn: &Connection, month: &str) -> Result<MonthSummary, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, amount_cents, category_id, occurred_at, note, created_at, updated_at
             FROM expenses WHERE substr(occurred_at, 1, 7) = ?1
             ORDER BY occurred_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([month], |row| {
            Ok(Expense {
                id: row.get(0)?,
                amount_cents: row.get(1)?,
                category_id: row.get(2)?,
                occurred_at: row.get(3)?,
                note: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    let (count, total_cents) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(amount_cents), 0)
             FROM expenses WHERE substr(occurred_at, 1, 7) = ?1",
            [month],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .map_err(|e| e.to_string())?;
    Ok(MonthSummary {
        items,
        total_cents,
        count,
    })
}

// ---------- 分类管理 ----------

/// 同级下是否存在同名分类（一级分类因 UNIQUE 对 NULL 不生效，故代码层额外检查）
fn name_conflict(
    conn: &Connection,
    name: &str,
    parent_id: Option<i64>,
    exclude_id: i64,
) -> rusqlite::Result<bool> {
    match parent_id {
        None => conn.query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM categories
                 WHERE name = ?1 AND parent_id IS NULL AND id != ?2
             )",
            rusqlite::params![name, exclude_id],
            |row| row.get(0),
        ),
        Some(pid) => conn.query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM categories
                 WHERE name = ?1 AND parent_id = ?2 AND id != ?3
             )",
            rusqlite::params![name, pid, exclude_id],
            |row| row.get(0),
        ),
    }
}

pub fn add_category(
    conn: &Connection,
    name: &str,
    parent_id: Option<i64>,
) -> Result<Category, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("分类名称不能为空".into());
    }
    if let Some(pid) = parent_id {
        let is_level1: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM categories WHERE id = ?1 AND parent_id IS NULL)",
                [pid],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if !is_level1 {
            return Err("父分类必须是一级大类".into());
        }
    }
    if name_conflict(conn, name, parent_id, 0).map_err(|e| e.to_string())? {
        return Err("同级下已存在同名分类".into());
    }
    conn.execute(
        "INSERT INTO categories (name, parent_id) VALUES (?1, ?2)",
        rusqlite::params![name, parent_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(Category {
        id: conn.last_insert_rowid(),
        name: name.to_string(),
        parent_id,
    })
}

pub fn rename_category(conn: &Connection, id: i64, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("分类名称不能为空".into());
    }
    let parent_id: Option<i64> = conn
        .query_row("SELECT parent_id FROM categories WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .map_err(|_| "分类不存在".to_string())?;
    if name_conflict(conn, name, parent_id, id).map_err(|e| e.to_string())? {
        return Err("同级下已存在同名分类".into());
    }
    conn.execute(
        "UPDATE categories SET name = ?1 WHERE id = ?2",
        rusqlite::params![name, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_category(conn: &Connection, id: i64) -> Result<(), String> {
    let children: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM categories WHERE parent_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if children > 0 {
        return Err("该分类下还有二级小类，请先删除它们".into());
    }
    let used: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM expenses WHERE category_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if used > 0 {
        return Err(format!("该分类下有 {used} 笔账单，无法删除"));
    }
    conn.execute("DELETE FROM categories WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------- 统计 ----------

/// 生成以 end_month 结尾、往前共 n 个月的 "YYYY-MM" 列表（升序）
fn month_strings_before(end_month: &str, n: usize) -> Vec<String> {
    let year: i32 = end_month
        .get(0..4)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2026);
    let month: i32 = end_month
        .get(5..7)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let mut out = Vec::with_capacity(n);
    let (mut y, mut m) = (year, month);
    for _ in 0..n {
        out.push(format!("{y:04}-{m:02}"));
        m -= 1;
        if m == 0 {
            m = 12;
            y -= 1;
        }
    }
    out.reverse();
    out
}

/// 近 n 个月（含 end_month）每月支出合计，无数据的月份补 0
pub fn month_trend(conn: &Connection, end_month: &str, n: usize) -> Result<Vec<MonthTotal>, String> {
    let months = month_strings_before(end_month, n);
    let mut totals = HashMap::<String, (i64, i64)>::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT substr(occurred_at, 1, 7) AS m, COALESCE(SUM(amount_cents), 0), COUNT(*)
                 FROM expenses WHERE substr(occurred_at, 1, 7) <= ?1 GROUP BY m",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([end_month], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for r in rows {
            let (m, t, c) = r.map_err(|e| e.to_string())?;
            totals.insert(m, (t, c));
        }
    }
    Ok(months
        .into_iter()
        .map(|m| {
            let (t, c) = totals.get(&m).copied().unwrap_or((0, 0));
            MonthTotal {
                month: m,
                total_cents: t,
                count: c,
            }
        })
        .collect())
}

/// 某月各一级分类支出统计（按金额降序）
pub fn category_stats(conn: &Connection, month: &str) -> Result<Vec<CategoryStat>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(p.id, c.id), COALESCE(p.name, c.name), SUM(e.amount_cents), COUNT(*)
             FROM expenses e
             JOIN categories c ON c.id = e.category_id
             LEFT JOIN categories p ON p.id = c.parent_id
             WHERE substr(e.occurred_at, 1, 7) = ?1
             GROUP BY COALESCE(p.id, c.id)
             ORDER BY SUM(e.amount_cents) DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([month], |row| {
            Ok(CategoryStat {
                id: row.get(0)?,
                name: row.get(1)?,
                total_cents: row.get(2)?,
                count: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

/// 某月二级分类支出排行（前 limit 名）
pub fn child_ranking(
    conn: &Connection,
    month: &str,
    limit: i64,
) -> Result<Vec<CategoryStat>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name, SUM(e.amount_cents), COUNT(*)
             FROM expenses e
             JOIN categories c ON c.id = e.category_id
             WHERE c.parent_id IS NOT NULL AND substr(e.occurred_at, 1, 7) = ?1
             GROUP BY c.id
             ORDER BY SUM(e.amount_cents) DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![month, limit], |row| {
            Ok(CategoryStat {
                id: row.get(0)?,
                name: row.get(1)?,
                total_cents: row.get(2)?,
                count: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

/// 某年 12 个月的支出与全年合计
pub fn year_report(conn: &Connection, year: &str) -> Result<YearReport, String> {
    let mut totals = HashMap::<String, (i64, i64)>::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT substr(occurred_at, 6, 2), COALESCE(SUM(amount_cents), 0), COUNT(*)
                 FROM expenses WHERE substr(occurred_at, 1, 4) = ?1
                 GROUP BY substr(occurred_at, 6, 2)",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([year], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for r in rows {
            let (m, t, c) = r.map_err(|e| e.to_string())?;
            totals.insert(m, (t, c));
        }
    }
    let items = (1..=12)
        .map(|m| {
            let key = format!("{m:02}");
            let (t, c) = totals.get(&key).copied().unwrap_or((0, 0));
            MonthStat {
                month: format!("{year}-{key}"),
                total_cents: t,
                count: c,
            }
        })
        .collect::<Vec<_>>();
    let (total_cents, count) = items.iter().fold(
        (0i64, 0i64),
        |(ts, cs), it| (ts + it.total_cents, cs + it.count),
    );
    Ok(YearReport {
        year: year.to_string(),
        items,
        total_cents,
        count,
    })
}

// ---------- 数据管理 ----------

/// CSV 字段转义：含逗号 / 引号 / 换行时用双引号包裹，内部引号翻倍
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// 导出账单为 CSV（UTF-8 带 BOM，Excel 打开中文不乱码）；month 为 None 时导出全部
/// 返回导出的笔数
pub fn export_expenses_csv(
    conn: &Connection,
    path: &Path,
    month: Option<&str>,
) -> Result<usize, String> {
    let mut stmt = conn
        .prepare(
            "SELECT e.occurred_at, COALESCE(p.name, c.name), c.name, e.amount_cents, e.note
             FROM expenses e
             JOIN categories c ON c.id = e.category_id
             LEFT JOIN categories p ON p.id = c.parent_id
             WHERE (?1 IS NULL OR substr(e.occurred_at, 1, 7) = ?1)
             ORDER BY e.occurred_at, e.id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![month], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut out: Vec<u8> = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    let mut push = |line: String| out.extend_from_slice(line.as_bytes());
    push("时间,一级分类,二级分类,金额(元),备注\n".to_string());
    let mut n = 0usize;
    for r in rows {
        let (at, parent, child, cents, note) = r.map_err(|e| e.to_string())?;
        let yuan = format!("{}.{:02}", cents / 100, cents % 100);
        push(format!(
            "{},{},{},{},{}\n",
            at,
            csv_escape(&parent),
            csv_escape(&child),
            yuan,
            csv_escape(&note)
        ));
        n += 1;
    }
    std::fs::write(path, out).map_err(|e| format!("写入文件失败：{e}"))?;
    Ok(n)
}

/// 在线备份当前数据库到目标文件（SQLite 备份 API，事务安全）
pub fn backup_db(conn: &Connection, target: &Path) -> Result<(), String> {
    conn.backup(rusqlite::MAIN_DB, target, None).map_err(|e| {
        // 备份失败时清掉可能残留的空壳文件，避免后续被误当备份
        let _ = std::fs::remove_file(target);
        e.to_string()
    })
}

/// 从备份文件恢复：校验文件头 → 自动安全备份当前库 → SQLite 在线恢复写回 → 结构校验
pub fn restore_db(conn: &mut Connection, db_path: &Path, source: &Path) -> Result<(), String> {
    let head = std::fs::read(source).map_err(|e| format!("无法读取备份文件：{e}"))?;
    if head.len() < 16 || &head[..16] != b"SQLite format 3\0" {
        return Err(format!(
            "所选文件不是有效的数据库备份：{}（请选择由本应用「备份数据」生成的 .db 文件）",
            source.display()
        ));
    }
    // 安全兜底：恢复前自动把当前数据另存一份
    let safety = db_path.with_file_name("pre-restore-auto.db");
    conn.backup(rusqlite::MAIN_DB, &safety, None)
        .map_err(|e| e.to_string())?;
    // SQLite 在线恢复：把备份内容写回当前连接（无需关闭 / 覆盖文件，无 Windows 文件锁问题）
    conn.restore(
        rusqlite::MAIN_DB,
        source,
        None::<fn(rusqlite::backup::Progress)>,
    )
    .map_err(|e| format!("恢复失败：{e}"))?;
    // 结构校验：补全表结构、必要时重新预置分类
    ensure_schema(conn).map_err(|e| format!("恢复后的数据库结构校验失败：{e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::NewExpense;

    #[test]
    fn time_format_ok() {
        assert!(valid_time("2026-08-23 14:52"));
        assert!(valid_time("2026-01-01 00:00"));
        assert!(valid_time("2026-12-31 23:59"));
    }

    #[test]
    fn time_format_rejects_bad_input() {
        assert!(!valid_time(""));
        assert!(!valid_time("2026-08-23"));
        assert!(!valid_time("2026/08/23 14:52"));
        assert!(!valid_time("2026-08-23 14:52:00"));
        assert!(!valid_time("2026-8-3 4:5"));
    }

    #[test]
    fn month_strings_cross_year() {
        let list = month_strings_before("2026-03", 14);
        assert_eq!(list.len(), 14);
        assert_eq!(list[0], "2025-02");
        assert_eq!(list[13], "2026-03");

        let jan = month_strings_before("2026-01", 3);
        assert_eq!(jan, vec!["2025-11", "2025-12", "2026-01"]);
    }

    #[test]
    fn stats_workflow() {
        let path = std::env::temp_dir().join(format!("blackhorse_test_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        {
            let conn = init(&path).expect("init");
            let add = |cents: i64, cat: i64, at: &str| {
                create_expense(
                    &conn,
                    &NewExpense {
                        amount_cents: cents,
                        category_id: cat,
                        occurred_at: at.to_string(),
                        note: String::new(),
                    },
                )
                .expect("create expense");
            };
            // 内置分类 id：1=餐饮食品，2=早餐，3=午餐
            add(100, 2, "2025-12-01 09:00");
            add(200, 3, "2026-01-15 10:00");
            add(500, 2, "2026-01-20 11:00");

            let trend = month_trend(&conn, "2026-01", 3).expect("trend");
            assert_eq!(trend.len(), 3);
            assert_eq!(trend[0].month, "2025-11");
            assert_eq!(trend[0].total_cents, 0);
            assert_eq!(trend[1].total_cents, 100);
            assert_eq!(trend[2].month, "2026-01");
            assert_eq!(trend[2].total_cents, 700);
            assert_eq!(trend[2].count, 2);

            let cats = category_stats(&conn, "2026-01").expect("cats");
            assert_eq!(cats.len(), 1);
            assert_eq!(cats[0].name, "餐饮食品");
            assert_eq!(cats[0].total_cents, 700);

            let rank = child_ranking(&conn, "2026-01", 10).expect("rank");
            assert_eq!(rank.len(), 2);
            assert_eq!(rank[0].name, "早餐");
            assert_eq!(rank[0].total_cents, 500);

            let report = year_report(&conn, "2026").expect("report");
            assert_eq!(report.total_cents, 700);
            assert_eq!(report.count, 2);
            assert_eq!(report.items[0].total_cents, 700); // 2026-01
            assert_eq!(report.items[11].total_cents, 0); // 2026-12
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn data_management_workflow() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("blackhorse_test_dm_{}.db", std::process::id()));
        let csv_path = dir.join(format!("blackhorse_test_{}.csv", std::process::id()));
        let backup_path = dir.join(format!("blackhorse_test_bak_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&csv_path);
        let _ = std::fs::remove_file(&backup_path);
        {
            let mut conn = init(&path).expect("init");
            create_expense(
                &conn,
                &NewExpense {
                    amount_cents: 1234,
                    category_id: 2,
                    occurred_at: "2026-08-23 12:00".to_string(),
                    note: "午饭,加餐\"好吃\"".to_string(),
                },
            )
            .expect("create");

            // 导出 CSV：BOM + 转义 + 笔数
            let n = export_expenses_csv(&conn, &csv_path, Some("2026-08")).expect("export");
            assert_eq!(n, 1);
            let content = std::fs::read(&csv_path).expect("read csv");
            assert!(content.starts_with(&[0xEF, 0xBB, 0xBF]));
            let text = String::from_utf8(content[3..].to_vec()).expect("utf8");
            assert!(text.starts_with("时间,一级分类,二级分类,金额(元),备注"));
            assert!(text.contains("12.34"));
            assert!(text.contains("\"午饭,加餐\"\"好吃\"\"\""));

            // 备份 → 再记一笔 → 恢复 → 只剩 1 笔
            backup_db(&conn, &backup_path).expect("backup");
            create_expense(
                &conn,
                &NewExpense {
                    amount_cents: 999,
                    category_id: 3,
                    occurred_at: "2026-08-23 13:00".to_string(),
                    note: String::new(),
                },
            )
            .expect("create2");
            restore_db(&mut conn, &path, &backup_path).expect("restore");
            let trend = month_trend(&conn, "2026-08", 1).expect("trend after restore");
            assert_eq!(trend[0].count, 1);
            assert_eq!(trend[0].total_cents, 1234);
        }
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&csv_path);
        let _ = std::fs::remove_file(&backup_path);
        let _ = std::fs::remove_file(dir.join(format!("blackhorse_test_dm_{}.db", std::process::id()))
            .with_file_name("pre-restore-auto.db"));
    }

    #[test]
    fn backup_restore_with_chinese_paths() {
        let dir = std::env::temp_dir().join("黑马记账测试目录");
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("黑马数据.db");
        let backup_path = dir.join("黑马备份-20260823.db");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup_path);
        {
            let mut conn = init(&path).expect("init");
            create_expense(
                &conn,
                &NewExpense {
                    amount_cents: 888,
                    category_id: 2,
                    occurred_at: "2026-08-23 18:00".to_string(),
                    note: "中文路径测试".to_string(),
                },
            )
            .expect("create");
            backup_db(&conn, &backup_path).expect("backup to chinese path");
            restore_db(&mut conn, &path, &backup_path).expect("restore from chinese path");
            let trend = month_trend(&conn, "2026-08", 1).expect("trend");
            assert_eq!(trend[0].count, 1);
        }
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup_path);
        let _ = std::fs::remove_file(dir.join("pre-restore-auto.db"));
    }

    #[test]
    fn restore_after_app_workload() {
        // 模拟真实应用的工作负载（大量查询）后执行恢复，排查连接状态问题
        let dir = std::env::temp_dir();
        let path = dir.join(format!("blackhorse_repro_{}.db", std::process::id()));
        let backup_path = dir.join(format!("blackhorse_repro_bak_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup_path);
        {
            let mut conn = init(&path).expect("init");
            for _ in 0..3 {
                list_categories(&conn).expect("cats");
                list_expenses(&conn, "2026-08").expect("list");
                month_trend(&conn, "2026-08", 12).expect("trend");
                category_stats(&conn, "2026-08").expect("stats");
                child_ranking(&conn, "2026-08", 100).expect("rank");
                year_report(&conn, "2026").expect("report");
            }
            create_expense(
                &conn,
                &NewExpense {
                    amount_cents: 111,
                    category_id: 2,
                    occurred_at: "2026-08-23 09:00".to_string(),
                    note: String::new(),
                },
            )
            .expect("create");
            backup_db(&conn, &backup_path).expect("backup");
            create_expense(
                &conn,
                &NewExpense {
                    amount_cents: 222,
                    category_id: 3,
                    occurred_at: "2026-08-23 10:00".to_string(),
                    note: String::new(),
                },
            )
            .expect("create2");
            restore_db(&mut conn, &path, &backup_path).expect("restore after workload");
            let trend = month_trend(&conn, "2026-08", 1).expect("trend after");
            assert_eq!(trend[0].count, 1);
            assert_eq!(trend[0].total_cents, 111);
        }
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&backup_path);
        let _ = std::fs::remove_file(dir.join("pre-restore-auto.db"));
    }
}
