<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle } from '$lib/components/forms';

  const config = configStore.config;
  const isDirty = configStore.isDirty;
  const saving = configStore.saving;
  const validationErrors = configStore.validationErrors;

  function getError(field: string): string {
    const found = $validationErrors.find(e => e.field === field);
    return found?.message || '';
  }

  async function handleSave() {
    await configStore.save();
  }
</script>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/config/document-flows">Document Flows</a>
      <span class="separator">/</span>
      <span>Order to Cash</span>
    </div>
    <div class="header-row">
      <div>
        <h1>Order to Cash (O2C)</h1>
        <p>Configure the sales order-to-cash receipt document flow</p>
      </div>
      <div class="header-actions">
        {#if $isDirty}
          <button class="btn-secondary" onclick={() => configStore.reset()}>
            Discard
          </button>
        {/if}
        <button
          class="btn-primary"
          onclick={handleSave}
          disabled={$saving || !$isDirty}
        >
          {$saving ? 'Saving...' : 'Save Changes'}
        </button>
      </div>
    </div>
  </header>

  {#if $config?.document_flows?.o2c}
    {@const o2c = $config.document_flows.o2c}
    <div class="page-content">
      <div class="flow-diagram">
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
              <path d="M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1z" />
            </svg>
          </div>
          <span class="step-label">Sales Order</span>
        </div>
        <div class="flow-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
          <span class="days-label">{o2c.average_so_to_delivery_days} days</span>
        </div>
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M1 3h15v13H1zM16 8h4l3 3v5h-7V8z" />
              <circle cx="5.5" cy="18.5" r="2.5" />
              <circle cx="18.5" cy="18.5" r="2.5" />
            </svg>
          </div>
          <span class="step-label">Delivery</span>
        </div>
        <div class="flow-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
          <span class="days-label">{o2c.average_delivery_to_invoice_days} days</span>
        </div>
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <path d="M14 2v6h6M16 13H8M16 17H8M10 9H8" />
            </svg>
          </div>
          <span class="step-label">Customer Invoice</span>
        </div>
        <div class="flow-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
          <span class="days-label">{o2c.average_invoice_to_receipt_days} days</span>
        </div>
        <div class="flow-step">
          <div class="step-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
            </svg>
          </div>
          <span class="step-label">Cash Receipt</span>
        </div>
      </div>

      <FormSection title="General Settings" description="Enable or disable O2C flow generation">
        {#snippet children()}
          <Toggle
            bind:checked={o2c.enabled}
            label="Enable O2C Flow"
            description="Generate complete Order-to-Cash document chains"
          />
        {/snippet}
      </FormSection>

      <FormSection title="Credit & Risk" description="Configure credit check and bad debt behavior">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Credit Check Failure Rate"
              htmlFor="credit-fail"
              helpText="Percentage of orders that fail credit check"
              error={getError('document_flows.o2c.credit_check_failure_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="credit-fail"
                    bind:value={o2c.credit_check_failure_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.credit_check_failure_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Bad Debt Rate"
              htmlFor="bad-debt"
              helpText="Percentage of invoices written off as uncollectible"
              error={getError('document_flows.o2c.bad_debt_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="bad-debt"
                    bind:value={o2c.bad_debt_rate}
                    min="0"
                    max="1"
                    step="0.001"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.bad_debt_rate * 100).toFixed(1)}%</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Fulfillment Settings" description="Configure delivery and returns behavior">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Partial Shipment Rate"
              htmlFor="partial-ship"
              helpText="Percentage of orders with partial shipments"
              error={getError('document_flows.o2c.partial_shipment_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="partial-ship"
                    bind:value={o2c.partial_shipment_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.partial_shipment_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Return Rate"
              htmlFor="return-rate"
              helpText="Percentage of orders with customer returns"
              error={getError('document_flows.o2c.return_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="return-rate"
                    bind:value={o2c.return_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.return_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Timing Configuration" description="Average days between document steps">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="SO to Delivery"
              htmlFor="so-to-delivery"
              helpText="Average days from sales order to delivery"
              error={getError('document_flows.o2c.average_so_to_delivery_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="so-to-delivery"
                    bind:value={o2c.average_so_to_delivery_days}
                    min="0"
                    max="365"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Delivery to Invoice"
              htmlFor="delivery-to-invoice"
              helpText="Average days from delivery to customer invoice"
              error={getError('document_flows.o2c.average_delivery_to_invoice_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="delivery-to-invoice"
                    bind:value={o2c.average_delivery_to_invoice_days}
                    min="0"
                    max="365"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Invoice to Receipt"
              htmlFor="invoice-to-receipt"
              helpText="Average days from invoice to cash receipt"
              error={getError('document_flows.o2c.average_invoice_to_receipt_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="invoice-to-receipt"
                    bind:value={o2c.average_invoice_to_receipt_days}
                    min="0"
                    max="365"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Cash Discounts" description="Early payment discount settings">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Eligible Rate"
              htmlFor="discount-eligible"
              helpText="Percentage of invoices eligible for early payment discount"
              error={getError('document_flows.o2c.cash_discount.eligible_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="discount-eligible"
                    bind:value={o2c.cash_discount.eligible_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.cash_discount.eligible_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Discount Taken Rate"
              htmlFor="discount-taken"
              helpText="Percentage of customers who take the discount"
              error={getError('document_flows.o2c.cash_discount.taken_rate')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="discount-taken"
                    bind:value={o2c.cash_discount.taken_rate}
                    min="0"
                    max="1"
                    step="0.01"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.cash_discount.taken_rate * 100).toFixed(0)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Discount Percentage"
              htmlFor="discount-percent"
              helpText="Discount percentage for early payment"
              error={getError('document_flows.o2c.cash_discount.discount_percent')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="discount-percent"
                    bind:value={o2c.cash_discount.discount_percent}
                    min="0"
                    max="1"
                    step="0.005"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">{(o2c.cash_discount.discount_percent * 100).toFixed(1)}%</span>
                </div>
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Discount Window"
              htmlFor="discount-days"
              helpText="Days within which discount must be taken"
              error={getError('document_flows.o2c.cash_discount.discount_days')}
            >
              {#snippet children()}
                <div class="input-with-suffix">
                  <input
                    type="number"
                    id="discount-days"
                    bind:value={o2c.cash_discount.discount_days}
                    min="0"
                    max="90"
                    disabled={!o2c.enabled}
                  />
                  <span class="suffix">days</span>
                </div>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Line Items" description="Configure SO line count distribution">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Minimum Lines"
              htmlFor="min-lines"
              helpText="Minimum number of lines per SO"
              error={getError('document_flows.o2c.line_count_distribution.min_lines')}
            >
              {#snippet children()}
                <input
                  type="number"
                  id="min-lines"
                  bind:value={o2c.line_count_distribution.min_lines}
                  min="1"
                  max="100"
                  disabled={!o2c.enabled}
                />
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Maximum Lines"
              htmlFor="max-lines"
              helpText="Maximum number of lines per SO"
              error={getError('document_flows.o2c.line_count_distribution.max_lines')}
            >
              {#snippet children()}
                <input
                  type="number"
                  id="max-lines"
                  bind:value={o2c.line_count_distribution.max_lines}
                  min="1"
                  max="100"
                  disabled={!o2c.enabled}
                />
              {/snippet}
            </FormGroup>

            <FormGroup
              label="Most Common (Mode)"
              htmlFor="mode-lines"
              helpText="Most common line count"
              error={getError('document_flows.o2c.line_count_distribution.mode_lines')}
            >
              {#snippet children()}
                <input
                  type="number"
                  id="mode-lines"
                  bind:value={o2c.line_count_distribution.mode_lines}
                  min="1"
                  max="100"
                  disabled={!o2c.enabled}
                />
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>
    </div>
  {:else}
    <div class="loading">
      <p>Loading configuration...</p>
    </div>
  {/if}
</div>

<style>
  .page {
    max-width: 900px;
  }

  .page-header {
    margin-bottom: var(--space-6);
  }

  .breadcrumb {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 0.8125rem;
    margin-bottom: var(--space-3);
  }

  .breadcrumb a {
    color: var(--color-accent);
    text-decoration: none;
  }

  .breadcrumb a:hover {
    text-decoration: underline;
  }

  .breadcrumb .separator {
    color: var(--color-text-muted);
  }

  .breadcrumb span:last-child {
    color: var(--color-text-secondary);
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .header-row h1 {
    margin-bottom: var(--space-1);
  }

  .header-actions {
    display: flex;
    gap: var(--space-2);
  }

  .page-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .flow-diagram {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-5);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    flex-wrap: wrap;
  }

  .flow-step {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
  }

  .step-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    color: var(--color-accent);
  }

  .step-icon svg {
    width: 24px;
    height: 24px;
  }

  .step-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-align: center;
    white-space: nowrap;
  }

  .flow-arrow {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-1);
    color: var(--color-text-muted);
  }

  .flow-arrow svg {
    width: 32px;
    height: 32px;
  }

  .days-label {
    font-size: 0.6875rem;
    font-family: var(--font-mono);
    color: var(--color-accent);
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
  }

  .input-with-suffix {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .input-with-suffix input {
    flex: 1;
  }

  .input-with-suffix .suffix {
    font-size: 0.8125rem;
    font-family: var(--font-mono);
    color: var(--color-text-secondary);
    min-width: 50px;
    text-align: right;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .form-grid {
      grid-template-columns: 1fr;
    }

    .header-row {
      flex-direction: column;
      gap: var(--space-4);
    }

    .flow-diagram {
      flex-direction: column;
    }

    .flow-arrow {
      transform: rotate(90deg);
    }
  }
</style>
