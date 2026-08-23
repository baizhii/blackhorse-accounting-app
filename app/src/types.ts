export interface Category {
  id: number;
  name: string;
  parentId: number | null;
}

export interface Expense {
  id: number;
  /** 金额，单位：分 */
  amountCents: number;
  categoryId: number;
  /** 消费时间，格式 "YYYY-MM-DD HH:MM" */
  occurredAt: string;
  note: string;
  createdAt: string;
  updatedAt: string;
}

export interface NewExpensePayload {
  amountCents: number;
  categoryId: number;
  occurredAt: string;
  note: string;
}

export interface MonthSummary {
  items: Expense[];
  totalCents: number;
  count: number;
}

export interface MonthTotal {
  month: string;
  totalCents: number;
  count: number;
}

export interface CategoryStat {
  id: number;
  name: string;
  totalCents: number;
  count: number;
}

export interface MonthStat {
  month: string;
  totalCents: number;
  count: number;
}

export interface YearReport {
  year: string;
  items: MonthStat[];
  totalCents: number;
  count: number;
}

/** 分类级联选择器的选项结构 */
export interface CascaderOption {
  id: number;
  name: string;
  children?: CascaderOption[];
}
