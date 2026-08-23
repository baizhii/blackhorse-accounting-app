<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useCategoriesStore } from "../stores/categories";
import { useExpensesStore } from "../stores/expenses";

const cats = useCategoriesStore();
const exps = useExpensesStore();

const dataPath = ref("");
const exportMonth = ref<string | null>(null);

/** 近 12 个月（含当前月，倒序），供导出范围选择 */
const recentMonths = computed(() => {
  const out: string[] = [];
  const d = new Date();
  for (let i = 0; i < 12; i++) {
    const dt = new Date(d.getFullYear(), d.getMonth() - i, 1);
    out.push(
      `${dt.getFullYear()}-${String(dt.getMonth() + 1).padStart(2, "0")}`
    );
  }
  return out;
});

function stamp(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}-${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}`;
}

onMounted(async () => {
  dataPath.value = await invoke<string>("data_file_path");
});

async function doExport() {
  const filePath = await save({
    defaultPath: `黑马记账-账单-${exportMonth.value ?? "全部"}.csv`,
    filters: [{ name: "CSV 文件", extensions: ["csv"] }],
  });
  if (!filePath) return;
  try {
    const n = await invoke<number>("export_expenses_csv", {
      path: filePath,
      month: exportMonth.value,
    });
    ElMessage.success(`已导出 ${n} 笔账单：${filePath}`);
  } catch (e) {
    ElMessage.error(`导出失败：${e}`);
  }
}

async function doBackup() {
  const filePath = await save({
    defaultPath: `黑马记账-备份-${stamp()}.db`,
    filters: [{ name: "数据库备份", extensions: ["db"] }],
  });
  if (!filePath) return;
  try {
    await invoke("backup_db", { target: filePath });
    ElMessage.success(`备份完成：${filePath}`);
  } catch (e) {
    ElMessage.error(`备份失败：${e}`);
  }
}

async function doRestore() {
  // 不设类型过滤，显示全部文件；后端会校验是否为有效备份并给出提示
  const filePath = await open({
    multiple: false,
    title: "选择备份文件",
  });
  if (!filePath) return;
  try {
    await ElMessageBox.confirm(
      "恢复将用备份文件覆盖当前全部账单数据。恢复前会自动把当前数据另存一份安全备份（pre-restore-auto.db，位于数据目录内）。确定继续？",
      "恢复确认",
      { type: "warning", confirmButtonText: "继续恢复", cancelButtonText: "取消" }
    );
  } catch {
    return; // 用户取消
  }
  try {
    await invoke("restore_db", { source: filePath });
    await Promise.all([cats.reload(), exps.load()]);
    ElMessage.success("恢复完成，数据已更新");
  } catch (e) {
    ElMessage.error(`恢复失败：${e}`);
  }
}

async function revealDir() {
  try {
    await revealItemInDir(dataPath.value);
  } catch (e) {
    ElMessage.error(`无法打开文件夹：${e}`);
  }
}
</script>

<template>
  <div class="page">
    <h2 class="page-title">数据管理</h2>

    <section class="card">
      <h3 class="card-title">导出账单（CSV）</h3>
      <p class="tip">
        导出为 CSV 文件，可用 Excel / WPS 直接打开，中文不乱码。<br />
        提示：若时间列显示为「######」，双击该列表头右边缘即可自动调整列宽（数据本身无损坏）。
      </p>
      <div class="row">
        <el-select v-model="exportMonth" placeholder="导出范围" class="scope-select">
          <el-option label="全部账单" :value="null" />
          <el-option v-for="m in recentMonths" :key="m" :label="m" :value="m" />
        </el-select>
        <el-button type="primary" @click="doExport">导出 CSV</el-button>
      </div>
    </section>

    <section class="card">
      <h3 class="card-title">数据备份与恢复</h3>
      <p class="tip">
        备份：把当前全部数据另存为一个文件，建议定期备份到其他位置（如 U 盘 / 网盘）。<br />
        恢复：用备份文件覆盖当前数据；恢复前会自动把当前数据另存一份安全备份（pre-restore-auto.db）。
      </p>
      <div class="row">
        <el-button @click="doBackup">💾 备份数据</el-button>
        <el-button type="danger" plain @click="doRestore">↩ 从备份恢复</el-button>
      </div>
    </section>

    <section class="card">
      <h3 class="card-title">数据文件位置</h3>
      <div class="row path-row">
        <code class="path">{{ dataPath }}</code>
        <el-button link type="primary" @click="revealDir">打开所在文件夹</el-button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.page {
  width: 100%;
  max-width: 720px;
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
  margin-bottom: 10px;
}

.tip {
  color: var(--muted);
  font-size: 12px;
  line-height: 1.7;
  margin-bottom: 12px;
}

.row {
  display: flex;
  gap: 12px;
  align-items: center;
}

.scope-select {
  width: 180px;
}

.path-row {
  flex-wrap: wrap;
}

.path {
  font-size: 12px;
  background: var(--hover);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 4px 8px;
  color: var(--text-2);
  word-break: break-all;
}
</style>
