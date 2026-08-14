"use client";

import { useMemo } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { useQuery } from "@tanstack/react-query";
import { DataGrid, type GridColDef } from "@mui/x-data-grid";
import { PageHeader, Button, ErrorState, EmptyState, FilterBar, NativeSelect } from "@/components/ui";
import { listCampaigns } from "@/lib/api/campaigns";

type CampaignRow = {
  id: string;
  name: string;
  status: string;
  sent_count?: number;
  delivered_count?: number;
};

export default function CampaignsPage() {
  const router = useRouter();
  const params = useSearchParams();
  const status = params.get("status") || "";
  const { data, isError, refetch, isLoading } = useQuery({
    queryKey: ["campaigns", status],
    queryFn: () => listCampaigns({ status: status || undefined, limit: 50 }),
  });
  const items: CampaignRow[] = data?.data || data?.items || data?.campaigns || [];

  const columns: GridColDef<CampaignRow>[] = useMemo(
    () => [
      { field: "name", headerName: "Name", flex: 1, minWidth: 160 },
      { field: "status", headerName: "Status", width: 130 },
      { field: "sent_count", headerName: "Sent", width: 100, valueGetter: (_v, row) => row.sent_count ?? 0 },
      { field: "delivered_count", headerName: "Delivered", width: 120, valueGetter: (_v, row) => row.delivered_count ?? 0 },
    ],
    []
  );

  return (
    <div className="space-y-6">
      <PageHeader
        title="Campaigns"
        description="Create, schedule, and launch bulk messaging."
        actions={<Button onClick={() => router.push("/campaigns/new")}>Create campaign</Button>}
      />
      <FilterBar>
        <NativeSelect
          value={status}
          onChange={(e) => router.push(e.target.value ? `/campaigns?status=${e.target.value}` : "/campaigns")}
        >
          <option value="">All statuses</option>
          <option value="draft">Drafts</option>
          <option value="scheduled">Scheduled</option>
          <option value="running">Running</option>
          <option value="completed">Completed</option>
        </NativeSelect>
      </FilterBar>
      {isError ? (
        <ErrorState onRetry={() => refetch()} />
      ) : isLoading ? null : !items.length ? (
        <EmptyState title="No campaigns" action={<Button onClick={() => router.push("/campaigns/new")}>Create campaign</Button>} />
      ) : (
        <div className="h-[480px] w-full rounded-xl border border-border bg-card">
          <DataGrid
            rows={items}
            columns={columns}
            disableRowSelectionOnClick
            onRowClick={(params) => router.push(`/campaigns/${params.id}`)}
            pageSizeOptions={[10, 25, 50]}
            initialState={{ pagination: { paginationModel: { pageSize: 25 } } }}
            sx={{ border: "none" }}
          />
        </div>
      )}
    </div>
  );
}
