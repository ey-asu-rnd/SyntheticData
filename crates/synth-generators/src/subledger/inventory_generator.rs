//! Inventory generator.

use chrono::NaiveDate;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use synth_core::models::subledger::inventory::{
    InventoryMovement, InventoryPosition, MovementType, PositionValuation, ReferenceDocType,
    StockStatus, ValuationMethod,
};
use synth_core::models::{JournalEntry, JournalEntryLine};

/// Configuration for inventory generation.
#[derive(Debug, Clone)]
pub struct InventoryGeneratorConfig {
    /// Default valuation method.
    pub default_valuation_method: ValuationMethod,
    /// Average unit cost.
    pub avg_unit_cost: Decimal,
    /// Unit cost variation.
    pub cost_variation: Decimal,
    /// Average movement quantity.
    pub avg_movement_quantity: Decimal,
    /// Quantity variation.
    pub quantity_variation: Decimal,
}

impl Default for InventoryGeneratorConfig {
    fn default() -> Self {
        Self {
            default_valuation_method: ValuationMethod::MovingAverage,
            avg_unit_cost: dec!(100),
            cost_variation: dec!(0.5),
            avg_movement_quantity: dec!(50),
            quantity_variation: dec!(0.8),
        }
    }
}

/// Generator for inventory transactions.
pub struct InventoryGenerator {
    config: InventoryGeneratorConfig,
    rng: ChaCha8Rng,
    movement_counter: u64,
    position_counter: u64,
}

impl InventoryGenerator {
    /// Creates a new inventory generator.
    pub fn new(config: InventoryGeneratorConfig, rng: ChaCha8Rng) -> Self {
        Self {
            config,
            rng,
            movement_counter: 0,
            position_counter: 0,
        }
    }

    /// Generates an initial inventory position.
    pub fn generate_position(
        &mut self,
        company_code: &str,
        plant: &str,
        storage_location: &str,
        material_id: &str,
        material_description: &str,
        initial_quantity: Decimal,
        unit_cost: Option<Decimal>,
        currency: &str,
    ) -> InventoryPosition {
        self.position_counter += 1;
        let cost = unit_cost.unwrap_or_else(|| self.generate_unit_cost());
        let total_value = (initial_quantity * cost).round_dp(2);

        InventoryPosition {
            position_id: format!("POS{:08}", self.position_counter),
            company_code: company_code.to_string(),
            plant: plant.to_string(),
            storage_location: storage_location.to_string(),
            material_id: material_id.to_string(),
            material_description: material_description.to_string(),
            quantity_on_hand: initial_quantity,
            quantity_reserved: Decimal::ZERO,
            quantity_available: initial_quantity,
            quantity_in_transit: Decimal::ZERO,
            quantity_in_quality: Decimal::ZERO,
            base_unit: "EA".to_string(),
            valuation: PositionValuation {
                method: self.config.default_valuation_method.clone(),
                unit_cost: cost,
                total_value,
                standard_cost: Some(cost),
                last_purchase_price: Some(cost),
                moving_average_price: Some(cost),
            },
            currency: currency.to_string(),
            last_movement_date: None,
            last_count_date: None,
            abc_indicator: None,
            stock_status: StockStatus::Unrestricted,
            minimum_stock: Some(dec!(10)),
            maximum_stock: Some(dec!(1000)),
            reorder_point: Some(dec!(50)),
        }
    }

    /// Generates a goods receipt (inventory increase).
    pub fn generate_goods_receipt(
        &mut self,
        position: &InventoryPosition,
        receipt_date: NaiveDate,
        quantity: Decimal,
        unit_cost: Decimal,
        po_number: Option<&str>,
    ) -> (InventoryMovement, JournalEntry) {
        self.movement_counter += 1;
        let movement_id = format!("INVMV{:08}", self.movement_counter);

        let total_value = (quantity * unit_cost).round_dp(2);

        let movement = InventoryMovement {
            movement_id: movement_id.clone(),
            company_code: position.company_code.clone(),
            plant: position.plant.clone(),
            storage_location: position.storage_location.clone(),
            material_id: position.material_id.clone(),
            material_description: position.material_description.clone(),
            movement_type: MovementType::GoodsReceipt,
            movement_date: receipt_date,
            posting_date: receipt_date,
            quantity,
            base_unit: position.base_unit.clone(),
            unit_cost,
            total_value,
            currency: position.currency.clone(),
            reference_doc_type: po_number.map(|_| ReferenceDocType::PurchaseOrder),
            reference_doc_number: po_number.map(|s| s.to_string()),
            reference_doc_line: None,
            cost_center: None,
            profit_center: None,
            gl_account: Some("1300".to_string()),
            movement_reason: Some("Goods Receipt from PO".to_string()),
            batch_number: Some(format!("BATCH{:06}", self.rng.gen::<u32>() % 1000000)),
            serial_numbers: None,
            special_stock_type: None,
            special_stock_number: None,
            created_by: "SYSTEM".to_string(),
            created_at: receipt_date,
            reversed: false,
            reversal_document: None,
        };

        let je = self.generate_goods_receipt_je(&movement);
        (movement, je)
    }

    /// Generates a goods issue (inventory decrease).
    pub fn generate_goods_issue(
        &mut self,
        position: &InventoryPosition,
        issue_date: NaiveDate,
        quantity: Decimal,
        cost_center: Option<&str>,
        production_order: Option<&str>,
    ) -> (InventoryMovement, JournalEntry) {
        self.movement_counter += 1;
        let movement_id = format!("INVMV{:08}", self.movement_counter);

        let unit_cost = position.valuation.unit_cost;
        let total_value = (quantity * unit_cost).round_dp(2);

        let (ref_type, ref_num) = if let Some(po) = production_order {
            (
                Some(ReferenceDocType::ProductionOrder),
                Some(po.to_string()),
            )
        } else {
            (None, None)
        };

        let movement = InventoryMovement {
            movement_id: movement_id.clone(),
            company_code: position.company_code.clone(),
            plant: position.plant.clone(),
            storage_location: position.storage_location.clone(),
            material_id: position.material_id.clone(),
            material_description: position.material_description.clone(),
            movement_type: MovementType::GoodsIssue,
            movement_date: issue_date,
            posting_date: issue_date,
            quantity,
            base_unit: position.base_unit.clone(),
            unit_cost,
            total_value,
            currency: position.currency.clone(),
            reference_doc_type: ref_type,
            reference_doc_number: ref_num,
            reference_doc_line: None,
            cost_center: cost_center.map(|s| s.to_string()),
            profit_center: None,
            gl_account: Some("1300".to_string()),
            movement_reason: Some("Goods Issue to Production".to_string()),
            batch_number: None,
            serial_numbers: None,
            special_stock_type: None,
            special_stock_number: None,
            created_by: "SYSTEM".to_string(),
            created_at: issue_date,
            reversed: false,
            reversal_document: None,
        };

        let je = self.generate_goods_issue_je(&movement);
        (movement, je)
    }

    /// Generates a stock transfer between locations.
    pub fn generate_transfer(
        &mut self,
        position: &InventoryPosition,
        transfer_date: NaiveDate,
        quantity: Decimal,
        to_plant: &str,
        to_storage_location: &str,
    ) -> (InventoryMovement, InventoryMovement, JournalEntry) {
        // Issue from source
        self.movement_counter += 1;
        let issue_id = format!("INVMV{:08}", self.movement_counter);

        // Receipt at destination
        self.movement_counter += 1;
        let receipt_id = format!("INVMV{:08}", self.movement_counter);

        let unit_cost = position.valuation.unit_cost;
        let total_value = (quantity * unit_cost).round_dp(2);

        let issue = InventoryMovement {
            movement_id: issue_id.clone(),
            company_code: position.company_code.clone(),
            plant: position.plant.clone(),
            storage_location: position.storage_location.clone(),
            material_id: position.material_id.clone(),
            material_description: position.material_description.clone(),
            movement_type: MovementType::TransferOut,
            movement_date: transfer_date,
            posting_date: transfer_date,
            quantity,
            base_unit: position.base_unit.clone(),
            unit_cost,
            total_value,
            currency: position.currency.clone(),
            reference_doc_type: Some(ReferenceDocType::StockTransfer),
            reference_doc_number: Some(receipt_id.clone()),
            reference_doc_line: None,
            cost_center: None,
            profit_center: None,
            gl_account: Some("1300".to_string()),
            movement_reason: Some(format!("Transfer to {}/{}", to_plant, to_storage_location)),
            batch_number: None,
            serial_numbers: None,
            special_stock_type: None,
            special_stock_number: None,
            created_by: "SYSTEM".to_string(),
            created_at: transfer_date,
            reversed: false,
            reversal_document: None,
        };

        let receipt = InventoryMovement {
            movement_id: receipt_id.clone(),
            company_code: position.company_code.clone(),
            plant: to_plant.to_string(),
            storage_location: to_storage_location.to_string(),
            material_id: position.material_id.clone(),
            material_description: position.material_description.clone(),
            movement_type: MovementType::TransferIn,
            movement_date: transfer_date,
            posting_date: transfer_date,
            quantity,
            base_unit: position.base_unit.clone(),
            unit_cost,
            total_value,
            currency: position.currency.clone(),
            reference_doc_type: Some(ReferenceDocType::StockTransfer),
            reference_doc_number: Some(issue_id.clone()),
            reference_doc_line: None,
            cost_center: None,
            profit_center: None,
            gl_account: Some("1300".to_string()),
            movement_reason: Some(format!(
                "Transfer from {}/{}",
                position.plant, position.storage_location
            )),
            batch_number: None,
            serial_numbers: None,
            special_stock_type: None,
            special_stock_number: None,
            created_by: "SYSTEM".to_string(),
            created_at: transfer_date,
            reversed: false,
            reversal_document: None,
        };

        // For intra-company transfer, no GL impact unless different plants have different valuations
        let je = self.generate_transfer_je(&issue, &receipt);

        (issue, receipt, je)
    }

    /// Generates an inventory adjustment.
    pub fn generate_adjustment(
        &mut self,
        position: &InventoryPosition,
        adjustment_date: NaiveDate,
        quantity_change: Decimal,
        reason: &str,
    ) -> (InventoryMovement, JournalEntry) {
        self.movement_counter += 1;
        let movement_id = format!("INVMV{:08}", self.movement_counter);

        let movement_type = if quantity_change > Decimal::ZERO {
            MovementType::InventoryAdjustmentIn
        } else {
            MovementType::InventoryAdjustmentOut
        };

        let unit_cost = position.valuation.unit_cost;
        let total_value = (quantity_change.abs() * unit_cost).round_dp(2);

        let movement = InventoryMovement {
            movement_id: movement_id.clone(),
            company_code: position.company_code.clone(),
            plant: position.plant.clone(),
            storage_location: position.storage_location.clone(),
            material_id: position.material_id.clone(),
            material_description: position.material_description.clone(),
            movement_type,
            movement_date: adjustment_date,
            posting_date: adjustment_date,
            quantity: quantity_change.abs(),
            base_unit: position.base_unit.clone(),
            unit_cost,
            total_value,
            currency: position.currency.clone(),
            reference_doc_type: Some(ReferenceDocType::PhysicalInventory),
            reference_doc_number: Some(format!("PI{:08}", self.movement_counter)),
            reference_doc_line: None,
            cost_center: None,
            profit_center: None,
            gl_account: Some("1300".to_string()),
            movement_reason: Some(reason.to_string()),
            batch_number: None,
            serial_numbers: None,
            special_stock_type: None,
            special_stock_number: None,
            created_by: "SYSTEM".to_string(),
            created_at: adjustment_date,
            reversed: false,
            reversal_document: None,
        };

        let je = self.generate_adjustment_je(&movement, quantity_change > Decimal::ZERO);
        (movement, je)
    }

    fn generate_unit_cost(&mut self) -> Decimal {
        let base = self.config.avg_unit_cost;
        let variation = base * self.config.cost_variation;
        let random: f64 = self.rng.gen_range(-1.0..1.0);
        (base + variation * Decimal::try_from(random).unwrap_or_default())
            .max(dec!(1))
            .round_dp(2)
    }

    fn generate_goods_receipt_je(&self, movement: &InventoryMovement) -> JournalEntry {
        let mut je = JournalEntry::new(
            format!("JE-{}", movement.movement_id),
            movement.company_code.clone(),
            movement.posting_date,
            format!("Goods Receipt {}", movement.material_id),
        );

        // Debit Inventory
        je.add_line(JournalEntryLine {
            line_number: 1,
            account_code: "1300".to_string(),
            account_description: Some("Inventory".to_string()),
            debit_amount: movement.total_value,
            credit_amount: Decimal::ZERO,
            cost_center: movement.cost_center.clone(),
            profit_center: movement.profit_center.clone(),
            project_code: None,
            reference: Some(movement.movement_id.clone()),
            assignment: Some(movement.material_id.clone()),
            text: Some(movement.material_description.clone()),
            quantity: Some(movement.quantity),
            unit: Some(movement.base_unit.clone()),
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        // Credit GR/IR Clearing
        je.add_line(JournalEntryLine {
            line_number: 2,
            account_code: "2100".to_string(),
            account_description: Some("GR/IR Clearing".to_string()),
            debit_amount: Decimal::ZERO,
            credit_amount: movement.total_value,
            cost_center: None,
            profit_center: None,
            project_code: None,
            reference: movement.reference_doc_number.clone(),
            assignment: None,
            text: None,
            quantity: None,
            unit: None,
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        je
    }

    fn generate_goods_issue_je(&self, movement: &InventoryMovement) -> JournalEntry {
        let mut je = JournalEntry::new(
            format!("JE-{}", movement.movement_id),
            movement.company_code.clone(),
            movement.posting_date,
            format!("Goods Issue {}", movement.material_id),
        );

        // Debit Cost of Goods Sold or WIP
        let debit_account =
            if movement.reference_doc_type == Some(ReferenceDocType::ProductionOrder) {
                "1350".to_string() // WIP
            } else {
                "5100".to_string() // COGS
            };

        je.add_line(JournalEntryLine {
            line_number: 1,
            account_code: debit_account,
            account_description: Some("Cost of Goods Sold".to_string()),
            debit_amount: movement.total_value,
            credit_amount: Decimal::ZERO,
            cost_center: movement.cost_center.clone(),
            profit_center: movement.profit_center.clone(),
            project_code: None,
            reference: Some(movement.movement_id.clone()),
            assignment: Some(movement.material_id.clone()),
            text: Some(movement.material_description.clone()),
            quantity: Some(movement.quantity),
            unit: Some(movement.base_unit.clone()),
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        // Credit Inventory
        je.add_line(JournalEntryLine {
            line_number: 2,
            account_code: "1300".to_string(),
            account_description: Some("Inventory".to_string()),
            debit_amount: Decimal::ZERO,
            credit_amount: movement.total_value,
            cost_center: None,
            profit_center: None,
            project_code: None,
            reference: Some(movement.movement_id.clone()),
            assignment: Some(movement.material_id.clone()),
            text: None,
            quantity: Some(movement.quantity),
            unit: Some(movement.base_unit.clone()),
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        je
    }

    fn generate_transfer_je(
        &self,
        issue: &InventoryMovement,
        _receipt: &InventoryMovement,
    ) -> JournalEntry {
        // For intra-company transfer with same valuation, this might be a memo entry
        // or could involve plant-specific inventory accounts
        let mut je = JournalEntry::new(
            format!("JE-XFER-{}", issue.movement_id),
            issue.company_code.clone(),
            issue.posting_date,
            format!("Stock Transfer {}", issue.material_id),
        );

        // Debit Inventory at destination (using same account for simplicity)
        je.add_line(JournalEntryLine {
            line_number: 1,
            account_code: "1300".to_string(),
            account_description: Some("Inventory - Destination".to_string()),
            debit_amount: issue.total_value,
            credit_amount: Decimal::ZERO,
            cost_center: None,
            profit_center: None,
            project_code: None,
            reference: Some(issue.movement_id.clone()),
            assignment: Some(issue.material_id.clone()),
            text: None,
            quantity: Some(issue.quantity),
            unit: Some(issue.base_unit.clone()),
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        // Credit Inventory at source
        je.add_line(JournalEntryLine {
            line_number: 2,
            account_code: "1300".to_string(),
            account_description: Some("Inventory - Source".to_string()),
            debit_amount: Decimal::ZERO,
            credit_amount: issue.total_value,
            cost_center: None,
            profit_center: None,
            project_code: None,
            reference: Some(issue.movement_id.clone()),
            assignment: Some(issue.material_id.clone()),
            text: None,
            quantity: Some(issue.quantity),
            unit: Some(issue.base_unit.clone()),
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        je
    }

    fn generate_adjustment_je(
        &self,
        movement: &InventoryMovement,
        is_increase: bool,
    ) -> JournalEntry {
        let mut je = JournalEntry::new(
            format!("JE-{}", movement.movement_id),
            movement.company_code.clone(),
            movement.posting_date,
            format!("Inventory Adjustment {}", movement.material_id),
        );

        if is_increase {
            // Debit Inventory
            je.add_line(JournalEntryLine {
                line_number: 1,
                account_code: "1300".to_string(),
                account_description: Some("Inventory".to_string()),
                debit_amount: movement.total_value,
                credit_amount: Decimal::ZERO,
                cost_center: None,
                profit_center: None,
                project_code: None,
                reference: Some(movement.movement_id.clone()),
                assignment: Some(movement.material_id.clone()),
                text: Some(movement.movement_reason.clone().unwrap_or_default()),
                quantity: Some(movement.quantity),
                unit: Some(movement.base_unit.clone()),
                tax_code: None,
                trading_partner: None,
                value_date: None,
            });

            // Credit Inventory Adjustment Account
            je.add_line(JournalEntryLine {
                line_number: 2,
                account_code: "4950".to_string(),
                account_description: Some("Inventory Adjustments".to_string()),
                debit_amount: Decimal::ZERO,
                credit_amount: movement.total_value,
                cost_center: movement.cost_center.clone(),
                profit_center: None,
                project_code: None,
                reference: Some(movement.movement_id.clone()),
                assignment: None,
                text: None,
                quantity: None,
                unit: None,
                tax_code: None,
                trading_partner: None,
                value_date: None,
            });
        } else {
            // Debit Inventory Adjustment Account (expense)
            je.add_line(JournalEntryLine {
                line_number: 1,
                account_code: "6950".to_string(),
                account_description: Some("Inventory Shrinkage".to_string()),
                debit_amount: movement.total_value,
                credit_amount: Decimal::ZERO,
                cost_center: movement.cost_center.clone(),
                profit_center: None,
                project_code: None,
                reference: Some(movement.movement_id.clone()),
                assignment: None,
                text: Some(movement.movement_reason.clone().unwrap_or_default()),
                quantity: None,
                unit: None,
                tax_code: None,
                trading_partner: None,
                value_date: None,
            });

            // Credit Inventory
            je.add_line(JournalEntryLine {
                line_number: 2,
                account_code: "1300".to_string(),
                account_description: Some("Inventory".to_string()),
                debit_amount: Decimal::ZERO,
                credit_amount: movement.total_value,
                cost_center: None,
                profit_center: None,
                project_code: None,
                reference: Some(movement.movement_id.clone()),
                assignment: Some(movement.material_id.clone()),
                text: None,
                quantity: Some(movement.quantity),
                unit: Some(movement.base_unit.clone()),
                tax_code: None,
                trading_partner: None,
                value_date: None,
            });
        }

        je
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_generate_position() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = InventoryGenerator::new(InventoryGeneratorConfig::default(), rng);

        let position = generator.generate_position(
            "1000",
            "PLANT01",
            "WH01",
            "MAT001",
            "Raw Material A",
            dec!(100),
            None,
            "USD",
        );

        assert_eq!(position.quantity_on_hand, dec!(100));
        assert!(position.valuation.unit_cost > Decimal::ZERO);
    }

    #[test]
    fn test_generate_goods_receipt() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = InventoryGenerator::new(InventoryGeneratorConfig::default(), rng);

        let position = generator.generate_position(
            "1000",
            "PLANT01",
            "WH01",
            "MAT001",
            "Raw Material A",
            dec!(100),
            Some(dec!(50)),
            "USD",
        );

        let (movement, je) = generator.generate_goods_receipt(
            &position,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            dec!(50),
            dec!(50),
            Some("PO001"),
        );

        assert_eq!(movement.movement_type, MovementType::GoodsReceipt);
        assert_eq!(movement.quantity, dec!(50));
        assert!(je.is_balanced());
    }

    #[test]
    fn test_generate_goods_issue() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = InventoryGenerator::new(InventoryGeneratorConfig::default(), rng);

        let position = generator.generate_position(
            "1000",
            "PLANT01",
            "WH01",
            "MAT001",
            "Raw Material A",
            dec!(100),
            Some(dec!(50)),
            "USD",
        );

        let (movement, je) = generator.generate_goods_issue(
            &position,
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            dec!(30),
            Some("CC100"),
            None,
        );

        assert_eq!(movement.movement_type, MovementType::GoodsIssue);
        assert_eq!(movement.quantity, dec!(30));
        assert!(je.is_balanced());
    }
}
