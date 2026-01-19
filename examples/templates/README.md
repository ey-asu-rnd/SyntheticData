# Example Templates

This directory contains example template files for regional and sector-specific data generation.

## Available Templates

| Template | Region | Sector | Description |
|----------|--------|--------|-------------|
| `german_manufacturing.yaml` | Germany (de) | Manufacturing | German industrial manufacturing with authentic naming |
| `japanese_technology.yaml` | Japan (jp) | Technology | Japanese tech/electronics companies |
| `british_financial_services.yaml` | UK (gb) | Financial Services | City of London banking and insurance |
| `brazilian_retail.yaml` | Brazil (br) | Retail | Brazilian retail chains and e-commerce |
| `indian_healthcare.yaml` | India (in) | Healthcare | Indian hospitals and pharmaceutical companies |

## Template Structure

Each template file follows this structure:

```yaml
metadata:
  name: "Template Name"
  version: "1.0"
  region: "xx"           # ISO country code
  sector: "industry"     # Industry sector
  author: "Author"
  description: "Description"

person_names:
  cultures:
    culture_name:
      male_first_names: [...]
      female_first_names: [...]
      last_names: [...]

vendor_names:
  categories:
    category_name: [...]

customer_names:
  industries:
    industry_name: [...]

material_descriptions:
  by_type:
    type_name: [...]

asset_descriptions:
  by_category:
    category_name: [...]

line_item_descriptions:
  p2p:
    account_type: [...]
  o2c:
    account_type: [...]

header_text_templates:
  by_process:
    process_name: [...]
```

## Usage

### CLI Usage

```bash
# Use a specific template file
synth-data generate --templates ./examples/templates/german_manufacturing.yaml --output ./output

# Use all templates in a directory (they will be merged)
synth-data generate --templates ./examples/templates/ --output ./output
```

### Programmatic Usage

```rust
use synth_core::templates::loader::{TemplateLoader, MergeStrategy};
use std::path::Path;

// Load a single template
let template = TemplateLoader::load_from_file(
    Path::new("examples/templates/german_manufacturing.yaml")
)?;

// Load all templates from directory (merges them)
let merged = TemplateLoader::load_from_directory(
    Path::new("examples/templates/")
)?;

// Validate a template
let errors = TemplateLoader::validate(&template);
if !errors.is_empty() {
    for err in errors {
        eprintln!("Validation error: {}", err);
    }
}
```

## Creating Custom Templates

### Step 1: Copy an existing template

```bash
cp examples/templates/german_manufacturing.yaml my_custom_template.yaml
```

### Step 2: Edit the metadata

```yaml
metadata:
  name: "My Custom Template"
  version: "1.0"
  region: "us"
  sector: "technology"
  author: "Your Name"
  description: "Custom template for US tech companies"
```

### Step 3: Customize the content

Add your own names, descriptions, and terminology that matches your regional and sector requirements.

### Step 4: Validate the template

```bash
# The template will be validated when loading
cargo test -p synth-core test_load_example_templates
```

## Merge Strategies

When using multiple templates, you can control how they are merged:

| Strategy | Behavior |
|----------|----------|
| `Replace` | The overlay template completely replaces the base |
| `Extend` (default) | Items from overlay are added to base (lists extended) |
| `MergePreferFile` | Overlay values replace base values for same keys |

## LLM-Generated Templates

These templates can be generated using LLMs for locale-specific flavor. Here's an example prompt:

```
Generate a YAML template for synthetic accounting data generation for a
[COUNTRY] [INDUSTRY] company. Include:

1. 20 male and 20 female first names common in [COUNTRY]
2. 25 family names/surnames common in [COUNTRY]
3. 10-15 vendor company names for [INDUSTRY] in [COUNTRY] style
4. 10-15 customer company names for [INDUSTRY] in [COUNTRY] style
5. Material descriptions relevant to [INDUSTRY]
6. Accounting line item descriptions in [LANGUAGE]
7. Document header text templates in [LANGUAGE]

Use the following YAML structure:
[paste template structure from above]
```

## Contents Summary

### german_manufacturing.yaml
- 20 male names, 20 female names, 25 surnames (German)
- Manufacturing vendors (Präzisionsteile GmbH, Industriekomponenten AG, etc.)
- Automotive, machinery, electronics customers
- Industrial material descriptions (Stahlblech, Aluminiumlegierung, etc.)
- German line item texts (Materialeinkauf, Lieferantenrechnung, etc.)

### japanese_technology.yaml
- 20 male names, 20 female names, 25 surnames (Japanese)
- Electronics and software vendors
- Consumer electronics, automotive tech, industrial customers
- Component descriptions (MLCC Capacitor, NAND Flash, etc.)
- Technology-focused line item texts

### british_financial_services.yaml
- 20 male names, 20 female names, 25 surnames (British)
- Technology, professional services, facilities vendors
- Banking, insurance, asset management customers
- Financial services materials (licenses, data services, etc.)
- City of London terminology (mark-to-market, impairment provision, etc.)

### brazilian_retail.yaml
- 20 male names, 20 female names, 25 surnames (Brazilian Portuguese)
- Distribution, manufacturing, services vendors
- Retail chains, e-commerce, wholesale customers
- Merchandise descriptions in Portuguese
- Brazilian retail terminology (Nota Fiscal, PIX, etc.)

### indian_healthcare.yaml
- 20 male names, 20 female names, 25 surnames (Indian)
- Pharmaceuticals, medical equipment, services vendors
- Hospitals, pharmacy chains, clinic customers
- Medical material descriptions
- Healthcare terminology (TPA settlement, UHID, etc.)
