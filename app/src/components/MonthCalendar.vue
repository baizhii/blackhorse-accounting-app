<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useCategoriesStore } from "../stores/categories";
import { centsToYuan } from "../utils/money";
import type { Expense, MonthSummary } from "../types";

const cats = useCategoriesStore();

const month = ref(currentMonth());
const items = ref<Expense[]>([]);
const loading = ref(false);
const selectedDay = ref<string | null>(null);

/** 每天的总支出与笔数，键为 "YYYY-MM-DD" */
const dayTotals = computed(() => {
  const map = new Map<string, { cents: number; count: number }>();
  for (const e of items.value) {
    const d = e.occurredAt.slice(0, 10);
    const t = map.get(d) ?? { cents: 0, count: 0 };
    t.cents += e.amountCents;
    t.count += 1;
    map.set(d, t);
  }
  return map;
});

const monthTotal = computed(() => items.value.reduce((s, e) => s + e.amountCents, 0));
const monthCount = computed(() => items.value.length);

const monthLabel = computed(() => {
  const [y, m] = month.value.split("-");
  return `${y} 年 ${Number(m)} 月`;
});

/** 周一开头的 7 列网格；单元格为 "YYYY-MM-DD" 或 null（空白占位） */
const cells = computed<(string | null)[]>(() => {
  const [y, m] = month.value.split("-").map(Number);
  const lead = (new Date(y, m - 1, 1).getDay() + 6) % 7;
  const days = new Date(y, m, 0).getDate();
  const out: (string | null)[] = [];
  for (let i = 0; i < lead; i++) out.push(null);
  for (let d = 1; d <= days; d++) {
    out.push(`${month.value}-${String(d).padStart(2, "0")}`);
  }
  while (out.length % 7 !== 0) out.push(null);
  return out;
});

const dayExpenses = computed(() =>
  selectedDay.value
    ? items.value.filter((e) => e.occurredAt.slice(0, 10) === selectedDay.value)
    : []
);
const dayTotal = computed(() => dayExpenses.value.reduce((s, e) => s + e.amountCents, 0));
const dialogVisible = computed({
  get: () => selectedDay.value !== null,
  set: (v) => {
    if (!v) selectedDay.value = null;
  },
});

function currentMonth(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}

function todayStr(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

function shiftMonth(delta: number) {
  const [y, m] = month.value.split("-").map(Number);
  const d = new Date(y, m - 1 + delta, 1);
  const p = (n: number) => String(n).padStart(2, "0");
  month.value = `${d.getFullYear()}-${p(d.getMonth() + 1)}`;
  load();
}

async function load() {
  loading.value = true;
  try {
    items.value = (await invoke<MonthSummary>("list_expenses", { month: month.value })).items;
  } finally {
    loading.value = false;
  }
}

function openDay(day: string) {
  if (dayTotals.value.has(day)) selectedDay.value = day;
}

onMounted(load);
</script>

<template>
  <section class="card cal-card">
    <div class="cal-head">
      <h3 class="card-title">日历</h3>
      <el-button-group size="small">
        <el-button @click="shiftMonth(-1)">‹</el-button>
        <el-button @click="month = currentMonth(); load()">今天</el-button>
        <el-button @click="shiftMonth(1)">›</el-button>
      </el-button-group>
      <span class="month-label">{{ monthLabel }}</span>
    </div>

    <div v-loading="loading">
      <div class="week-row">
        <div v-for="w in ['一', '二', '三', '四', '五', '六', '日']" :key="w" class="week-cell">
          {{ w }}
        </div>
      </div>
      <div class="grid">
        <div
          v-for="(cell, i) in cells"
          :key="i"
          class="cell"
          :class="{
            blank: cell === null,
            today: cell === todayStr(),
            clickable: cell !== null && dayTotals.has(cell),
          }"
          @click="cell && openDay(cell)"
        >
          <template v-if="cell">
            <div class="day-num">{{ Number(cell.slice(8)) }}</div>
            <div v-if="dayTotals.has(cell)" class="day-amount">
              ¥{{ centsToYuan(dayTotals.get(cell)?.cents ?? 0) }}
            </div>
          </template>
        </div>
      </div>
      <div class="month-total">
        本月 <b>{{ monthCount }}</b> 笔 ·
        <span class="total">¥{{ centsToYuan(monthTotal) }}</span>
      </div>
    </div>

    <el-dialog
      v-model="dialogVisible"
      :title="selectedDay ? selectedDay + ' 的账单（¥' + centsToYuan(dayTotal) + '）' : ''"
      width="560"
    >
      <el-table :data="dayExpenses" border size="small">
        <el-table-column label="时间" width="110">
          <template #default="{ row }">{{ row.occurredAt.slice(11) }}</template>
        </el-table-column>
        <el-table-column label="分类">
          <template #default="{ row }">
            <el-tag size="small">{{ cats.fullNameOf(row.categoryId) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="金额" width="110">
          <template #default="{ row }">
            <span class="money">¥{{ centsToYuan(row.amountCents) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="备注" prop="note" show-overflow-tooltip />
      </el-table>
    </el-dialog>
  </section>
</template>

<style scoped>
.cal-card {
  display: flex;
  flex-direction: column;
}

.cal-head {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.cal-head .card-title {
  margin-bottom: 0;
}

.month-label {
  margin-left: auto;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-2);
}

.week-row,
.grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
}

.week-row {
  border-bottom: 1px solid var(--border);
}

.week-cell {
  text-align: center;
  font-size: 12px;
  color: var(--muted);
  padding: 4px 0;
}

.cell {
  min-height: 62px;
  border-bottom: 1px solid var(--border);
  border-right: 1px solid var(--border);
  padding: 6px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.cell:nth-child(7n) {
  border-right: none;
}

.cell.blank {
  background: transparent;
}

.cell.clickable {
  cursor: pointer;
}

.cell.clickable:hover {
  background: var(--hover);
}

.day-num {
  font-size: 12px;
  color: var(--text-2);
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.cell.today .day-num {
  background: #409eff;
  color: #fff;
  border-radius: 999px;
}

.day-amount {
  font-size: 11px;
  color: var(--danger);
  font-variant-numeric: tabular-nums;
  font-weight: 500;
}

.month-total {
  margin-top: 10px;
  text-align: right;
  color: var(--text-2);
  font-size: 12px;
}

.total {
  color: var(--danger);
  font-weight: 600;
}

.money {
  font-variant-numeric: tabular-nums;
  font-weight: 500;
}
</style>
