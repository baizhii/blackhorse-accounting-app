use serde::{Deserialize, Serialize};

/// 两级分类：parent_id 为 NULL 表示一级大类，否则为二级小类
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
}

/// 一笔支出
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Expense {
    pub id: i64,
    /// 金额，单位：分。整数存储避免浮点误差
    pub amount_cents: i64,
    pub category_id: i64,
    /// 消费时间，格式 "YYYY-MM-DD HH:MM"
    pub occurred_at: String,
    pub note: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 新增 / 修改支出的入参
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewExpense {
    pub amount_cents: i64,
    pub category_id: i64,
    pub occurred_at: String,
    pub note: String,
}

/// 某月的账单列表与汇总
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthSummary {
    pub items: Vec<Expense>,
    pub total_cents: i64,
    pub count: i64,
}

/// 单月支出合计（趋势图数据点）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthTotal {
    pub month: String,
    pub total_cents: i64,
    pub count: i64,
}

/// 某分类的支出统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryStat {
    pub id: i64,
    pub name: String,
    pub total_cents: i64,
    pub count: i64,
}

/// 年度报表中的单月数据
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthStat {
    pub month: String,
    pub total_cents: i64,
    pub count: i64,
}

/// 年度汇总报表
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YearReport {
    pub year: String,
    pub items: Vec<MonthStat>,
    pub total_cents: i64,
    pub count: i64,
}
