<script lang="ts">
	import { page } from "$app/stores";
	import { connected, runtimeRunning } from "$lib/stores/websocket";
	import { pendingCount } from "$lib/stores/approval";
	import { theme } from "$lib/stores/theme";
	import { updateAvailable, installUpdate } from "$lib/stores/update";
	import { persistGet, persistSet } from "$lib/stores/persistence";
	import { onMount } from "svelte";
	import {
		LayoutDashboard,
		Activity,
		CheckCircle,
		FileText,
		Target,
		Compass,
		DollarSign,
		Settings,
		ChevronLeft,
		ChevronRight,
		Zap,
		Sun,
		Moon,
		Download,
		Search,
		PenLine,
		Shield,
	} from "lucide-svelte";

	let collapsed = $state(false);
	let updating = $state(false);

	onMount(async () => {
		collapsed = await persistGet('sidebar_collapsed', false);
	});

	function toggleCollapsed() {
		collapsed = !collapsed;
		persistSet('sidebar_collapsed', collapsed);
	}

	async function handleUpdate() {
		updating = true;
		await installUpdate();
		updating = false;
	}

	const navItems = [
		{ href: "/", label: "Dashboard", icon: LayoutDashboard },
		{ href: "/activity", label: "Activity", icon: Activity },
		{ href: "/approval", label: "Approval", icon: CheckCircle },
		{ href: "/content", label: "Content", icon: FileText },
		{ href: "/drafts", label: "Drafts", icon: PenLine },
		{ href: "/discovery", label: "Discovery", icon: Search },
		{ href: "/targets", label: "Targets", icon: Target },
		{ href: "/strategy", label: "Strategy", icon: Compass },
		{ href: "/costs", label: "Costs", icon: DollarSign },
		{ href: "/mcp", label: "MCP", icon: Shield },
		{ href: "/settings", label: "Settings", icon: Settings },
	];

	function isActive(href: string, pathname: string): boolean {
		if (href === "/") return pathname === "/";
		return pathname.startsWith(href);
	}
</script>

<aside class="sidebar" class:collapsed>
	<div class="sidebar-header">
		{#if !collapsed}
			<div class="logo">
				<Zap size={20} strokeWidth={2.5} />
				<span class="logo-text">Tuitbot</span>
			</div>
		{:else}
			<div class="logo logo-collapsed">
				<Zap size={20} strokeWidth={2.5} />
			</div>
		{/if}
	</div>

	<nav class="sidebar-nav">
		{#each navItems as item}
			<a
				href={item.href}
				class="nav-link"
				class:active={isActive(item.href, $page.url.pathname)}
				title={collapsed ? item.label : undefined}
			>
				<item.icon size={18} />
				{#if !collapsed}
					<span>{item.label}</span>
					{#if item.href === '/approval' && $pendingCount > 0}
						<span class="nav-badge">{$pendingCount}</span>
					{/if}
				{/if}
			</a>
		{/each}
	</nav>

	<div class="sidebar-footer">
		{#if $updateAvailable}
			<button
				class="update-btn"
				onclick={handleUpdate}
				disabled={updating}
				title="Update available — click to install"
			>
				<Download size={14} />
				{#if !collapsed}
					<span>{updating ? 'Installing...' : 'Update Available'}</span>
				{/if}
			</button>
		{/if}

		<div
			class="status-row"
			title={$connected ? "Server connected" : "Server disconnected"}
		>
			<span class="status-dot" class:online={$connected}></span>
			{#if !collapsed}
				<span class="status-text">
					{#if $runtimeRunning}
						Running
					{:else if $connected}
						Connected
					{:else}
						Disconnected
					{/if}
				</span>
			{/if}
		</div>

		<div class="footer-actions">
			<button
				class="action-btn"
				onclick={() => theme.toggle()}
				title={$theme === "dark"
					? "Switch to light mode"
					: "Switch to dark mode"}
			>
				{#if $theme === "dark"}
					<Moon size={16} />
				{:else}
					<Sun size={16} />
				{/if}
				{#if !collapsed}
					<span>{$theme === "dark" ? "Dark" : "Light"}</span>
				{/if}
			</button>

			<button
				class="action-btn"
				onclick={toggleCollapsed}
				title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
			>
				{#if collapsed}
					<ChevronRight size={16} />
				{:else}
					<ChevronLeft size={16} />
				{/if}
			</button>
		</div>
	</div>
</aside>

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		width: 220px;
		min-height: 100vh;
		background-color: var(--color-surface);
		border-right: 1px solid var(--color-border-subtle);
		transition: width 0.2s ease;
		user-select: none;
	}

	.sidebar.collapsed {
		width: 56px;
	}

	.sidebar-header {
		padding: 16px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 10px;
		color: var(--color-accent);
	}

	.logo-collapsed {
		justify-content: center;
	}

	.logo-text {
		font-size: 16px;
		font-weight: 700;
		letter-spacing: -0.02em;
		color: var(--color-text);
	}

	.sidebar-nav {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 8px;
		gap: 2px;
	}

	.nav-link {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 10px;
		border-radius: 6px;
		color: var(--color-text-muted);
		text-decoration: none;
		font-size: 13px;
		font-weight: 500;
		transition:
			background-color 0.15s ease,
			color 0.15s ease;
	}

	.sidebar.collapsed .nav-link {
		justify-content: center;
		padding: 8px;
	}

	.nav-link:hover {
		background-color: var(--color-surface-hover);
		color: var(--color-text);
	}

	.nav-link.active {
		background-color: var(--color-surface-active);
		color: var(--color-text);
	}

	.nav-link.active::before {
		content: "";
		position: absolute;
		left: 0;
		width: 3px;
		height: 20px;
		background-color: var(--color-accent);
		border-radius: 0 3px 3px 0;
	}

	.sidebar-footer {
		padding: 12px;
		border-top: 1px solid var(--color-border-subtle);
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.status-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 6px;
	}

	.sidebar.collapsed .status-row {
		justify-content: center;
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background-color: var(--color-text-subtle);
		flex-shrink: 0;
		transition: background-color 0.3s ease;
	}

	.status-dot.online {
		background-color: var(--color-success);
		box-shadow: 0 0 6px var(--color-success);
	}

	.status-text {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.footer-actions {
		display: flex;
		gap: 4px;
	}

	.action-btn {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		font-size: 12px;
		transition:
			background-color 0.15s ease,
			color 0.15s ease;
	}

	.action-btn:hover {
		background-color: var(--color-surface-hover);
		color: var(--color-text-muted);
	}

	.nav-badge {
		margin-left: auto;
		padding: 1px 7px;
		border-radius: 10px;
		background-color: var(--color-accent);
		color: white;
		font-size: 11px;
		font-weight: 700;
		line-height: 1.3;
	}

	.update-btn {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 7px 10px;
		border: none;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
		cursor: pointer;
		font-size: 12px;
		font-weight: 500;
		transition: background 0.15s;
	}

	.update-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 20%, transparent);
	}

	.update-btn:disabled {
		opacity: 0.7;
		cursor: wait;
	}
</style>
