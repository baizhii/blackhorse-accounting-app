<script setup lang="ts">
import { computed, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { useCategoriesStore } from "../stores/categories";
import { useExpensesStore } from "../stores/expenses";
import { centsToYuan, yuanToCents } from "../utils/money";
import type { Expense, NewExpensePayload } from "../types";

const cats = useCategoriesStore();
const exps = useExpensesStore();

// ---------- 筛选与排序 ----------
const keyword = ref("");
const filterParent = ref<number | null>(null);
const sortBy = ref<"time" | "amount">("time");

const filtered = computed(() => {
  let rows = exps.items;
  if (filterParent.value !== null) {
    const children = new Set(cats.childrenOf(filterParent.value).map((c) => c.id));
    rows = rows.filter((r) => r.categoryId === filterParent.value || children.has(r.categoryId));
  }
  const kw = keyword.value.trim().toLowerCase();
  if (kw) {
    rows = rows.filter((r) => r.note.toLowerCase().includes(kw));
  }
  const arr = [...rows];
  if (sortBy.value === "amount") {
    arr.sort((a, b) => b.amountCents - a.amountCents);
  } else {
    arr.sort((a, b) => b.occurredAt.localeCompare(a.occurredAt));
  }
  return arr;
});

// ---------- 编辑 ----------
const dialogVisible = ref(false);
const editingId = ref<number | null>(null);
const editForm = ref({
  amount: null as number | null,
  category: [] as number[],
  occurredAt: "",
  note: "",
});

function openEdit(row: Expense) {
  editingId.value = row.id;
  editForm.value = {
    amount: row.amountCents / 100,
    category: [cats.parentIdOf(row.categoryId), row.categoryId] as number[],
    occurredAt: row.occurredAt,
    note: row.note,
  };
  dialogVisible.value = true;
}

async function saveEdit() {
  if (editForm.value.amount === null || editForm.value.amount <= 0) {
    return ElMessage.warning("请输入正确的金额");
  }
  if (editForm.value.category.length < 2 || editingId.value === null) {
    return ElMessage.warning("请选择二级分类");
  }
  const payload: NewExpensePayload = {
    amountCents: yuanToCents(editForm.value.amount),
    categoryId: editForm.value.category[1],
    occurredAt: editForm.value.occurredAt,
    note: editForm.value.note.trim(),
  };
  try {
    await exps.update(editingId.value, payload);
    ElMessage.success("已更新");
    dialogVisible.value = false;
  } catch (e) {
    ElMessage.error(`更新失败：${e}`);
  }
}

async function removeRow(row: Expense) {
  try {
    await ElMessageBox.confirm(`确定删除这笔 ¥${centsToYuan(row.amountCents)} 的账单？`, "删除确认", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
    });
  } catch {
    return; // 用户取消
  }
  try {
    await exps.remove(row.id);
    ElMessage.success("已删除");
  } catch (e) {
    ElMessage.error(`删除失败：${e}`);
  }
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <h2 class="page-title">账单流水</h2>
      <div class="summary">
        本月共 <b>{{ exps.count }}</b> 笔，合计
        <span class="total">¥{{ centsToYuan(exps.totalCents) }}</span>
      </div>
    </div>

    <div class="toolbar">
      <div class="tb-group">
        <el-date-picker
          v-model="exps.month"
          type="month"
          format="YYYY 年 MM 月"
          value-format="YYYY-MM"
          placeholder="选择月份"
          @change="exps.load()"
        />
      </div>
      <div class="tb-group">
        <el-select
          v-model="filterParent"
          placeholder="按大类筛选"
          clearable
          class="filter-parent"
        >
          <el-option v-for="p in cats.parents" :key="p.id" :label="p.name" :value="p.id" />
        </el-select>
      </div>
      <div class="tb-group tb-search">
        <el-input
          v-model="keyword"
          placeholder="搜索备注…"
          clearable
          class="search"
        />
      </div>
      <div class="tb-group">
        <el-radio-group v-model="sortBy">
          <el-radio-button value="time">时间</el-radio-button>
          <el-radio-button value="amount">金额</el-radio-button>
        </el-radio-group>
      </div>
    </div>

    <el-table :data="filtered" border v-loading="exps.loading" empty-text="本月还没有账单，去「记一笔」吧" class="table">
      <el-table-column label="时间" width="150">
        <template #default="{ row }">{{ row.occurredAt }}</template>
      </el-table-column>
      <el-table-column label="分类">
        <template #default="{ row }">
          <el-tag size="small">{{ cats.fullNameOf(row.categoryId) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="金额" width="120">
        <template #default="{ row }">
          <span class="money">¥{{ centsToYuan(row.amountCents) }}</span>
        </template>
      </el-table-column>
      <el-table-column label="备注" prop="note" show-overflow-tooltip />
      <el-table-column label="操作" width="130">
        <template #default="{ row }">
          <el-button size="small" link type="primary" @click="openEdit(row)">编辑</el-button>
          <el-button size="small" link type="danger" @click="removeRow(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="dialogVisible" title="编辑账单" width="460">
      <el-form label-position="top">
        <el-form-item label="金额（元）" required>
          <el-input-number
            v-model="editForm.amount"
            :min="0.01"
            :max="99999999.99"
            :precision="2"
            :controls="false"
            class="edit-amount"
          />
        </el-form-item>
        <el-form-item label="分类" required>
          <el-cascader
            v-model="editForm.category"
            :options="cats.cascaderOptions"
            :props="{ value: 'id', label: 'name' }"
            class="edit-category"
          />
        </el-form-item>
        <el-form-item label="时间">
          <el-date-picker
            v-model="editForm.occurredAt"
            type="datetime"
            format="YYYY-MM-DD HH:mm"
            value-format="YYYY-MM-DD HH:mm"
            class="edit-time"
          />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="editForm.note" maxlength="200" show-word-limit />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="saveEdit">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.page-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: 16px;
}

.page-title {
  font-size: 18px;
  font-weight: 600;
}

.summary {
  color: var(--text-2);
  font-size: 14px;
}

.total {
  color: var(--danger);
  font-size: 18px;
  font-weight: 600;
}

.toolbar {
  display: flex;
  align-items: center;
  margin-bottom: 16px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 6px 0;
  flex-wrap: nowrap;
}

.tb-group {
  display: flex;
  align-items: center;
  padding: 0 16px;
  border-right: 1px solid var(--border);
}

.tb-group:last-child {
  border-right: none;
}

.tb-search {
  flex: 1;
  min-width: 0;
}

.filter-parent {
  width: 150px;
}

.search {
  width: 100%;
  max-width: 260px;
}

.table {
  background: var(--panel);
  border-radius: 8px;
  overflow: hidden;
}

.money {
  font-variant-numeric: tabular-nums;
  font-weight: 500;
}

.edit-amount,
.edit-category,
.edit-time {
  width: 100%;
}
</style>
