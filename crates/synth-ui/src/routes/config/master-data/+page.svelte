<script lang="ts">
  import { configStore } from '$lib/stores/config';

  const config = configStore.config;

  const entityTypes = [
    {
      key: 'vendors' as const,
      title: 'Vendors',
      description: 'Supplier and service provider master data',
      icon: 'M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M12 7a4 4 0 1 0-8 0 4 4 0 0 0 8 0zM23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75',
      href: '/config/master-data/vendors',
    },
    {
      key: 'customers' as const,
      title: 'Customers',
      description: 'Customer accounts and credit settings',
      icon: 'M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 7a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75',
      href: '/config/master-data/customers',
    },
    {
      key: 'materials' as const,
      title: 'Materials',
      description: 'Products, raw materials, and inventory items',
      icon: 'M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16zM3.27 6.96L12 12.01l8.73-5.05M12 22.08V12',
      href: '/config/master-data/materials',
    },
    {
      key: 'assets' as const,
      title: 'Fixed Assets',
      description: 'Capital equipment and depreciation settings',
      icon: 'M2 20h20M6 16V8a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v8M9 16v-6h6v6',
      href: '/config/master-data/assets',
    },
    {
      key: 'employees' as const,
      title: 'Employees',
      description: 'User accounts and approval hierarchies',
      icon: 'M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 7a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75',
      href: '/config/master-data/employees',
    },
  ];

  function getCount(key: 'vendors' | 'customers' | 'materials' | 'assets' | 'employees'): number {
    if (!$config?.master_data) return 0;
    const entity = $config.master_data[key];
    return entity?.count ?? 0;
  }
</script>

<div class="page">
  <header class="page-header">
    <div>
      <h1>Master Data</h1>
      <p>Configure entity counts and distribution settings</p>
    </div>
  </header>

  <div class="entity-grid">
    {#each entityTypes as entity}
      <a href={entity.href} class="entity-card">
        <div class="entity-icon">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d={entity.icon} />
          </svg>
        </div>
        <div class="entity-info">
          <h3 class="entity-title">{entity.title}</h3>
          <p class="entity-description">{entity.description}</p>
        </div>
        <div class="entity-count">
          <span class="count-value">{getCount(entity.key)}</span>
          <span class="count-label">entities</span>
        </div>
        <div class="entity-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18l6-6-6-6" />
          </svg>
        </div>
      </a>
    {/each}
  </div>

  <div class="info-section">
    <h2>About Master Data</h2>
    <p>
      Master data entities are the foundational records that transactions reference.
      Each entity type has its own distribution settings that control characteristics
      like payment terms, credit ratings, and behavior patterns.
    </p>
    <ul class="info-list">
      <li><strong>Vendors</strong> - Configure payment terms, vendor behavior (strict/flexible), and intercompany relationships</li>
      <li><strong>Customers</strong> - Set credit limits, payment behavior, and customer segments</li>
      <li><strong>Materials</strong> - Define material types, valuation methods, and bill of materials</li>
      <li><strong>Fixed Assets</strong> - Configure asset classes, depreciation methods, and useful life</li>
      <li><strong>Employees</strong> - Set up approval hierarchies, transaction limits, and department assignments</li>
    </ul>
  </div>
</div>

<style>
  .page {
    max-width: 1000px;
  }

  .page-header {
    margin-bottom: var(--space-6);
  }

  .page-header h1 {
    margin-bottom: var(--space-1);
  }

  .entity-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: var(--space-4);
    margin-bottom: var(--space-8);
  }

  .entity-card {
    display: grid;
    grid-template-columns: auto 1fr auto auto;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    text-decoration: none;
    color: inherit;
    transition: all var(--transition-fast);
  }

  .entity-card:hover {
    border-color: var(--color-accent);
    box-shadow: var(--shadow-md);
  }

  .entity-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    color: var(--color-accent);
  }

  .entity-icon svg {
    width: 24px;
    height: 24px;
  }

  .entity-info {
    min-width: 0;
  }

  .entity-title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-1);
  }

  .entity-description {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .entity-count {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    padding: var(--space-2) var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .count-value {
    font-family: var(--font-mono);
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .count-label {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .entity-arrow {
    width: 24px;
    height: 24px;
    color: var(--color-text-muted);
    transition: transform var(--transition-fast);
  }

  .entity-card:hover .entity-arrow {
    transform: translateX(4px);
    color: var(--color-accent);
  }

  .entity-arrow svg {
    width: 100%;
    height: 100%;
  }

  .info-section {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
  }

  .info-section h2 {
    font-size: 0.9375rem;
    font-weight: 600;
    margin-bottom: var(--space-3);
  }

  .info-section p {
    font-size: 0.875rem;
    margin-bottom: var(--space-4);
  }

  .info-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .info-list li {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    padding-left: var(--space-4);
    position: relative;
  }

  .info-list li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0.5em;
    width: 6px;
    height: 6px;
    background-color: var(--color-accent);
    border-radius: 50%;
  }

  .info-list li strong {
    color: var(--color-text-primary);
    font-weight: 600;
  }

  @media (max-width: 768px) {
    .entity-card {
      grid-template-columns: auto 1fr auto;
    }

    .entity-arrow {
      display: none;
    }
  }
</style>
