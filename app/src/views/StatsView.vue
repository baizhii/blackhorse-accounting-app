<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import * as echarts from "echarts";
import { invoke } from "@tauri-apps/api/core";
import { useCategoriesStore } from "../stores/categories";
import { centsToYuan } from "../utils/money";
import type { CategoryStat, MonthTotal, YearReport } from "../types";
import MonthCalendar from "../components/MonthCalendar.vue";

const cats = useCategoriesStore();

const trend = ref<MonthTotal[]>([]);
const breakdown = ref<CategoryStat[]>([]);
const ranking = ref<CategoryStat[]>([]);
const report = ref<YearReport | null>(null);

/** 二级分类排行的按大类筛选（null = 全部） */
const rankParent = ref<number | null>(null);

/** 排行按一级分类筛选后取前 10 */
const filteredRanking = computed(() => {
  let rows = ranking.value;
  if (rankParent.value !== null) {
    const ids = new Set(cats.childrenOf(rankParent.value).map((c) => c.id));
    rows = rows.filter((r) => ids.has(r.id));
  }
  return rows.slice(0, 10);
});

/** 排行图高度随条数自适应 */
const rankChartHeight = computed(
  () => `${Math.max(140, filteredRanking.value.length * 34 + 44)}px`
);

const statMonth = ref(currentMonth());
const reportYear = ref(String(new Date().getFullYear()));

const trendEl = ref<HTMLDivElement>();
const breakdownEl = ref<HTMLDivElement>();
const rankingEl = ref<HTMLDivElement>();

let trendChart: echarts.ECharts | null = null;
let breakdownChart: echarts.ECharts | null = null;
let rankingChart: echarts.ECharts | null = null;

const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");

function currentMonth(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}

/** 从应用 CSS 变量读取主题墨色（浅色 / 深色实时切换） */
function themeInk() {
  const s = getComputedStyle(document.documentElement);
  const pick = (v: string, fb: string) => s.getPropertyValue(v).trim() || fb;
  const dark = document.documentElement.classList.contains("dark");
  return {
    text: pick("--text", "#1f2329"),
    text2: pick("--text-2", "#4e5969"),
    muted: pick("--muted", "#8a919f"),
    border: pick("--border", "#e5e6eb"),
    panel: pick("--panel", "#ffffff"),
    accent: dark ? "#3987e5" : "#2a78d6",
  };
}

function hexToRgba(hex: string, alpha: number): string {
  const n = parseInt(hex.replace("#", ""), 16);
  return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${alpha})`;
}

/** 金额（分）→ 紧凑元文本，用于坐标轴刻度：1280000 → "1.3万" */
function compactYuan(cents: number): string {
  const y = cents / 100;
  if (y >= 10000) {
    const w = y / 10000;
    return `${w >= 100 ? Math.round(w) : w.toFixed(1).replace(/\.0$/, "")}万`;
  }
  if (y >= 100) return String(Math.round(y));
  return String(Number(y.toFixed(2)));
}

// ---------- 趋势图：单系列折线 ----------
function renderTrend() {
  if (!trendChart) return;
  const ink = themeInk();
  const data = trend.value.map((t) => t.totalCents);
  trendChart.setOption({
    grid: { left: 8, right: 16, top: 28, bottom: 8, containLabel: true },
    tooltip: {
      trigger: "axis",
      axisPointer: { type: "cross", lineStyle: { color: ink.border, width: 1 } },
      backgroundColor: ink.panel,
      borderColor: ink.border,
      textStyle: { color: ink.text, fontSize: 12 },
      valueFormatter: (v: any) => `¥${centsToYuan(v)}`,
    },
    xAxis: {
      type: "category",
      boundaryGap: false,
      data: trend.value.map((t) => t.month.slice(2).replace("-", "/")),
      axisLine: { lineStyle: { color: ink.border } },
      axisTick: { show: false },
      axisLabel: { color: ink.muted, fontSize: 11 },
    },
    yAxis: {
      type: "value",
      splitLine: { lineStyle: { color: ink.border, width: 1 } },
      axisLabel: { color: ink.muted, fontSize: 11, formatter: (v: number) => compactYuan(v) },
    },
    series: [
      {
        name: "月支出",
        type: "line",
        data,
        smooth: false,
        symbol: "circle",
        symbolSize: 8,
        lineStyle: { width: 2, color: ink.accent },
        itemStyle: { color: ink.accent, borderColor: ink.panel, borderWidth: 2 },
        areaStyle: { color: hexToRgba(ink.accent, 0.1) },
        label: {
          show: true,
          position: "top",
          color: ink.text2,
          fontSize: 11,
          formatter: (p: any) =>
            p.dataIndex === data.length - 1 ? `¥${centsToYuan(p.value)}` : "",
        },
      },
    ],
  });
}

// ---------- 横向条形图（占比 / 排行共用）：单色 + 端部数值标签 ----------
function barListOption(items: { name: string; totalCents: number; count: number }[]) {
  const ink = themeInk();
  const total = items.reduce((s, it) => s + it.totalCents, 0);
  return {
    grid: { left: 8, right: 128, top: 8, bottom: 8, containLabel: true },
    tooltip: {
      trigger: "item",
      backgroundColor: ink.panel,
      borderColor: ink.border,
      textStyle: { color: ink.text, fontSize: 12 },
      formatter: (p: any) => {
        const it = p.data as { name: string; totalCents: number; count: number };
        return `${it.name}<br/>¥${centsToYuan(it.totalCents)} · ${it.count} 笔`;
      },
    },
    xAxis: {
      type: "value",
      splitLine: { lineStyle: { color: ink.border, width: 1 } },
      axisLabel: { color: ink.muted, fontSize: 11, formatter: (v: number) => compactYuan(v) },
    },
    yAxis: {
      type: "category",
      data: items.map((it) => it.name).reverse(),
      axisLine: { lineStyle: { color: ink.border } },
      axisTick: { show: false },
      axisLabel: { color: ink.text2, fontSize: 12 },
    },
    series: [
      {
        type: "bar",
        data: items
          .map((it) => ({ ...it, value: it.totalCents }))
          .reverse(),
        barWidth: 16,
        itemStyle: { color: ink.accent, borderRadius: [0, 4, 4, 0] },
        label: {
          show: true,
          position: "right",
          color: ink.text2,
          fontSize: 11,
          formatter: (p: any) => {
            const v = p.value as number;
            const pct = total > 0 ? ((v / total) * 100).toFixed(1) : "0.0";
            return `¥${centsToYuan(v)} · ${pct}%`;
          },
        },
      },
    ],
  };
}

function renderBreakdown() {
  if (!breakdownChart) return;
  breakdownChart.setOption(
    barListOption(breakdown.value.map((it) => ({ name: it.name, totalCents: it.totalCents, count: it.count })))
  );
}

function renderRanking() {
  if (!rankingChart) return;
  rankingChart.setOption(
    barListOption(
      filteredRanking.value.map((it) => ({
        name: cats.fullNameOf(it.id),
        totalCents: it.totalCents,
        count: it.count,
      }))
    )
  );
}

// 排行数据或一级分类筛选变化时，重建 / 更新排行图（空数据显示空状态）
watch(filteredRanking, async (rows) => {
  await nextTick();
  if (rows.length) {
    if (!rankingChart && rankingEl.value) rankingChart = echarts.init(rankingEl.value);
    renderRanking();
    rankingChart?.resize();
  } else {
    rankingChart?.dispose();
    rankingChart = null;
  }
});

function rerenderAll() {
  renderTrend();
  renderBreakdown();
  renderRanking();
}

function onResize() {
  trendChart?.resize();
  breakdownChart?.resize();
  rankingChart?.resize();
}

// ---------- 数据加载 ----------
async function loadTrend() {
  trend.value = await invoke<MonthTotal[]>("month_trend", { endMonth: currentMonth() });
  if (!trendChart && trendEl.value) trendChart = echarts.init(trendEl.value);
  renderTrend();
}

async function loadBreakdown() {
  breakdown.value = await invoke<CategoryStat[]>("category_stats", { month: statMonth.value });
  ranking.value = await invoke<CategoryStat[]>("child_ranking", {
    month: statMonth.value,
    limit: 100,
  });
  await nextTick();
  if (breakdown.value.length) {
    if (!breakdownChart && breakdownEl.value) breakdownChart = echarts.init(breakdownEl.value);
  } else {
    breakdownChart?.dispose();
    breakdownChart = null;
  }
  renderBreakdown();
}

async function loadReport() {
  report.value = await invoke<YearReport>("year_report", { year: reportYear.value });
}

function percentOf(cents: number): string {
  const total = report.value?.totalCents ?? 0;
  if (total <= 0) return "—";
  return `${((cents / total) * 100).toFixed(1)}%`;
}

onMounted(async () => {
  await nextTick();
  if (trendEl.value) trendChart = echarts.init(trendEl.value);
  await Promise.all([loadTrend(), loadBreakdown(), loadReport()]);
  window.addEventListener("resize", onResize);
  darkQuery.addEventListener("change", rerenderAll);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", onResize);
  darkQuery.removeEventListener("change", rerenderAll);
  trendChart?.dispose();
  breakdownChart?.dispose();
  rankingChart?.dispose();
});
</script>

<template>
  <div class="page">
    <h2 class="page-title">统计分析</h2>

    <section class="card">
      <h3 class="card-title">近 12 个月支出趋势</h3>
      <div ref="trendEl" class="chart chart-trend"></div>
    </section>

    <div class="row-cards">
      <section class="card half">
        <div class="card-head">
          <h3 class="card-title">分类占比</h3>
          <el-date-picker
            v-model="statMonth"
            type="month"
            format="YYYY 年 MM 月"
            value-format="YYYY-MM"
            @change="loadBreakdown"
          />
        </div>
        <div v-if="breakdown.length" ref="breakdownEl" class="chart chart-half"></div>
        <el-empty v-else description="该月没有账单" :image-size="60" />
      </section>

      <section class="card half">
        <div class="card-head">
          <h3 class="card-title">二级分类排行 Top 10</h3>
          <el-select v-model="rankParent" placeholder="按大类筛选" clearable class="rank-filter">
            <el-option v-for="p in cats.parents" :key="p.id" :label="p.name" :value="p.id" />
          </el-select>
        </div>
        <div
          v-if="filteredRanking.length"
          ref="rankingEl"
          class="chart"
          :style="{ height: rankChartHeight }"
        ></div>
        <el-empty
          v-else
          :description="rankParent !== null ? '该分类下暂无账单' : '该月没有账单'"
          :image-size="60"
        />
      </section>
    </div>

    <div class="row-cards">
      <MonthCalendar class="half" />
      <section class="card half">
        <div class="card-head">
          <h3 class="card-title">年度汇总报表</h3>
          <el-date-picker
            v-model="reportYear"
            type="year"
            format="YYYY 年"
            value-format="YYYY"
            @change="loadReport"
          />
        </div>
        <el-table :data="report?.items ?? []" border size="small" class="report-table">
          <el-table-column label="月份" prop="month" width="110" />
          <el-table-column label="笔数" width="90">
            <template #default="{ row }">{{ row.count > 0 ? row.count : "—" }}</template>
          </el-table-column>
          <el-table-column label="金额">
            <template #default="{ row }">
              <span class="money">¥{{ centsToYuan(row.totalCents) }}</span>
            </template>
          </el-table-column>
          <el-table-column label="占比" width="100">
            <template #default="{ row }">{{ percentOf(row.totalCents) }}</template>
          </el-table-column>
        </el-table>
        <div class="report-total">
          全年合计：<b>{{ report?.count ?? 0 }}</b> 笔，共
          <span class="total">¥{{ centsToYuan(report?.totalCents ?? 0) }}</span>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.page {
  width: 100%;
}

.page-title {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 20px;
}

.card {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 18px 20px;
  margin-bottom: 16px;
}

.card-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
}

.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.card-head .card-title {
  margin-bottom: 0;
}

.row-cards {
  display: flex;
  gap: 16px;
  align-items: stretch;
}

.half {
  flex: 1;
  min-width: 0;
}

.rank-filter {
  width: 150px;
}

.chart-trend {
  height: 320px;
}

.chart-half {
  height: 360px;
}

.money {
  font-variant-numeric: tabular-nums;
  font-weight: 500;
}

.report-table {
  margin-bottom: 12px;
}

.report-total {
  text-align: right;
  color: var(--text-2);
  font-size: 14px;
}

.report-total .total {
  color: var(--danger);
  font-size: 16px;
  font-weight: 600;
}
</style>
