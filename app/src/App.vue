<script setup lang="ts">
import { onMounted, ref } from "vue";
import RecordView from "./views/RecordView.vue";
import ListView from "./views/ListView.vue";
import CategoryView from "./views/CategoryView.vue";
import StatsView from "./views/StatsView.vue";
import DataView from "./views/DataView.vue";
import { useCategoriesStore } from "./stores/categories";
import { useExpensesStore } from "./stores/expenses";

type ViewKey = "record" | "list" | "stats" | "category" | "data";

const views: { key: ViewKey; label: string; icon: string }[] = [
  { key: "record", label: "记一笔", icon: "✏️" },
  { key: "list", label: "账单流水", icon: "📋" },
  { key: "stats", label: "统计分析", icon: "📊" },
  { key: "category", label: "分类管理", icon: "🏷️" },
  { key: "data", label: "数据管理", icon: "💾" },
];

const current = ref<ViewKey>("record");

const cats = useCategoriesStore();
const exps = useExpensesStore();

onMounted(async () => {
  await cats.load();
  await exps.load();
});
</script>

<template>
  <div class="shell">
    <aside class="side">
      <div class="brand">
        <span class="logo">🐴</span>
        <div>
          <div class="brand-name">黑马记账</div>
          <div class="brand-sub">BlackHorse</div>
        </div>
      </div>
      <nav class="nav">
        <button
          v-for="v in views"
          :key="v.key"
          class="nav-item"
          :class="{ active: current === v.key }"
          @click="current = v.key"
        >
          <span class="nav-icon">{{ v.icon }}</span>
          {{ v.label }}
        </button>
      </nav>
      <div class="side-foot">v0.1.0 · M2</div>
    </aside>
    <main class="content">
      <RecordView v-if="current === 'record'" />
      <ListView v-else-if="current === 'list'" />
      <StatsView v-else-if="current === 'stats'" />
      <CategoryView v-else-if="current === 'category'" />
      <DataView v-else />
    </main>
  </div>
</template>

<style>
:root {
  color-scheme: light;
  --bg: #f5f6f8;
  --panel: #ffffff;
  --border: #e5e6eb;
  --text: #1f2329;
  --text-2: #4e5969;
  --muted: #8a919f;
  --hover: #f2f3f5;
  --active-bg: #e8f3ff;
  --danger: #d03050;
  /* 正文字体（用户选定：思源黑体）；标题使用系统字体栈 */
  --el-font-family: "Noto Sans SC", "Segoe UI", "Microsoft YaHei", "PingFang SC", sans-serif;
  --font-title: "Segoe UI", "Microsoft YaHei", "PingFang SC", sans-serif;
}

html.dark {
  color-scheme: dark;
  --bg: #141414;
  --panel: #1d1e1f;
  --border: #3f4042;
  --text: #e5e6eb;
  --text-2: #a3a6ad;
  --muted: #6b7280;
  --hover: #262727;
  --active-bg: rgba(64, 158, 255, 0.16);
  --danger: #f56c6c;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html,
body,
#app {
  height: 100%;
}

body {
  font-family: var(--el-font-family);
  background: var(--bg);
  color: var(--text);
  overflow: hidden;
}

/* 标题保持系统字体 */
.page-title,
.brand-name,
.card-title {
  font-family: var(--font-title);
}

.shell {
  display: flex;
  height: 100vh;
}

.side {
  width: 200px;
  background: var(--panel);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px 20px 16px;
}

.logo {
  font-size: 26px;
}

.brand-name {
  font-size: 16px;
  font-weight: 600;
}

.brand-sub {
  font-size: 11px;
  color: var(--muted);
}

.nav {
  flex: 1;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  font-size: 14px;
  color: var(--text-2);
  cursor: pointer;
  text-align: left;
  font-family: inherit;
}

.nav-item:hover {
  background: var(--hover);
}

.nav-item.active {
  background: var(--active-bg);
  color: #409eff;
  font-weight: 500;
}

.nav-icon {
  font-size: 15px;
}

.side-foot {
  padding: 14px 20px;
  font-size: 11px;
  color: var(--muted);
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
}
</style>
