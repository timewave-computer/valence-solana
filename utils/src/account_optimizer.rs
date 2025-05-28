// Account size optimization and rent efficiency utilities
// Helps minimize account sizes and optimize rent costs for Valence Solana programs



/// Rent optimization constants
pub const RENT_EXEMPT_THRESHOLD: u64 = 890_880; // Minimum lamports for rent exemption
pub const ACCOUNT_HEADER_SIZE: usize = 8; // Discriminator size
pub const PUBKEY_SIZE: usize = 32;
pub const BOOL_SIZE: usize = 1;
pub const U8_SIZE: usize = 1;
pub const U16_SIZE: usize = 2;
pub const U32_SIZE: usize = 4;
pub const U64_SIZE: usize = 8;
pub const I64_SIZE: usize = 8;
pub const VEC_HEADER_SIZE: usize = 4; // Vec length prefix

/// Account size calculator for different data types
pub struct AccountSizeCalculator;

impl AccountSizeCalculator {
    /// Calculate size for a string field
    pub fn string_size(max_length: usize) -> usize {
        VEC_HEADER_SIZE + max_length
    }
    
    /// Calculate size for a Vec<T> field
    pub fn vec_size<T>(max_items: usize) -> usize {
        VEC_HEADER_SIZE + (max_items * std::mem::size_of::<T>())
    }
    
    /// Calculate size for a Vec<Pubkey> field
    pub fn pubkey_vec_size(max_items: usize) -> usize {
        VEC_HEADER_SIZE + (max_items * PUBKEY_SIZE)
    }
    
    /// Calculate size for an Option<T> field
    pub fn option_size<T>() -> usize {
        1 + std::mem::size_of::<T>() // 1 byte for Some/None + T size
    }
    
    /// Calculate total account size including discriminator
    pub fn total_account_size(data_size: usize) -> usize {
        ACCOUNT_HEADER_SIZE + data_size
    }
    
    /// Calculate rent cost for an account of given size
    pub fn calculate_rent_cost(account_size: usize) -> u64 {
        // Simplified rent calculation - in practice would use Rent sysvar
        let base_rent = 19_055_441; // Base rent per byte-year
        let size_factor = account_size as u64;
        (base_rent * size_factor) / 365 / 24 / 3600 // Per second
    }
    
    /// Suggest optimal account size with padding for future growth
    pub fn suggest_optimal_size(current_size: usize, growth_factor: f32) -> usize {
        let padded_size = (current_size as f32 * (1.0 + growth_factor)) as usize;
        
        // Round up to nearest 8-byte boundary for alignment
        (padded_size + 7) & !7
    }
}

/// Account optimization strategies
pub struct AccountOptimizer;

impl AccountOptimizer {
    /// Optimize string storage by using fixed-size arrays where possible
    pub fn optimize_string_storage(
        strings: &[String],
        max_length_hint: Option<usize>,
    ) -> OptimizedStringStorage {
        let max_len = max_length_hint.unwrap_or_else(|| {
            strings.iter().map(|s| s.len()).max().unwrap_or(0)
        });
        
        // If all strings are short and similar length, use fixed arrays
        if max_len <= 32 && strings.iter().all(|s| s.len() <= 32) {
            OptimizedStringStorage::FixedArray(max_len)
        } else {
            OptimizedStringStorage::DynamicVec(max_len)
        }
    }
    
    /// Optimize Vec storage by pre-allocating based on usage patterns
    pub fn optimize_vec_storage<T>(
        typical_size: usize,
        max_size: usize,
        growth_pattern: GrowthPattern,
    ) -> VecOptimization {
        match growth_pattern {
            GrowthPattern::Static => VecOptimization {
                initial_capacity: typical_size,
                growth_strategy: GrowthStrategy::Fixed,
                max_capacity: typical_size,
            },
            GrowthPattern::Linear => VecOptimization {
                initial_capacity: typical_size,
                growth_strategy: GrowthStrategy::Linear(typical_size / 4),
                max_capacity: max_size,
            },
            GrowthPattern::Exponential => VecOptimization {
                initial_capacity: std::cmp::min(typical_size, 8),
                growth_strategy: GrowthStrategy::Exponential(2.0),
                max_capacity: max_size,
            },
        }
    }
    
    /// Pack boolean flags into bit fields to save space
    pub fn pack_boolean_flags(flag_count: usize) -> BooleanPacking {
        if flag_count <= 8 {
            BooleanPacking::U8(1)
        } else if flag_count <= 16 {
            BooleanPacking::U16(1)
        } else if flag_count <= 32 {
            BooleanPacking::U32(1)
        } else if flag_count <= 64 {
            BooleanPacking::U64(1)
        } else {
            // Use byte array for larger flag sets
            let byte_count = flag_count.div_ceil(8);
            BooleanPacking::ByteArray(byte_count)
        }
    }
    
    /// Optimize enum storage by using smallest possible integer type
    pub fn optimize_enum_storage(variant_count: usize) -> EnumStorage {
        if variant_count <= 256 {
            EnumStorage::U8
        } else if variant_count <= 65536 {
            EnumStorage::U16
        } else {
            EnumStorage::U32
        }
    }
}

/// String storage optimization strategies
#[derive(Debug, Clone)]
pub enum OptimizedStringStorage {
    FixedArray(usize),  // Use [u8; N] for short, fixed-length strings
    DynamicVec(usize),  // Use Vec<u8> for variable-length strings
}

/// Vec growth patterns for optimization
#[derive(Debug, Clone, Copy)]
pub enum GrowthPattern {
    Static,      // Size doesn't change after initialization
    Linear,      // Grows by fixed amount
    Exponential, // Doubles in size
}

/// Vec optimization configuration
#[derive(Debug, Clone)]
pub struct VecOptimization {
    pub initial_capacity: usize,
    pub growth_strategy: GrowthStrategy,
    pub max_capacity: usize,
}

/// Growth strategies for dynamic data structures
#[derive(Debug, Clone)]
pub enum GrowthStrategy {
    Fixed,                // No growth
    Linear(usize),        // Grow by fixed amount
    Exponential(f32),     // Multiply by factor
}

/// Boolean packing strategies
#[derive(Debug, Clone)]
pub enum BooleanPacking {
    U8(usize),           // Pack into u8 (up to 8 flags)
    U16(usize),          // Pack into u16 (up to 16 flags)
    U32(usize),          // Pack into u32 (up to 32 flags)
    U64(usize),          // Pack into u64 (up to 64 flags)
    ByteArray(usize),    // Use byte array for many flags
}

/// Enum storage optimization
#[derive(Debug, Clone, Copy)]
pub enum EnumStorage {
    U8,   // Up to 256 variants
    U16,  // Up to 65536 variants
    U32,  // Up to 4B variants
}

/// Account layout optimizer for specific Valence account types
pub struct ValenceAccountOptimizer;

impl ValenceAccountOptimizer {
    /// Optimize Authorization account layout
    pub fn optimize_authorization_account(
        max_label_length: usize,
        max_allowed_users: usize,
    ) -> AccountLayout {
        let mut layout = AccountLayout::new("Authorization");
        
        // Fixed fields
        layout.add_field("owner", PUBKEY_SIZE);
        layout.add_field("is_active", BOOL_SIZE);
        layout.add_field("permission_type", U8_SIZE); // Enum as u8
        layout.add_field("not_before", I64_SIZE);
        layout.add_field("expiration", AccountSizeCalculator::option_size::<i64>());
        layout.add_field("max_concurrent_executions", U32_SIZE);
        layout.add_field("priority", U8_SIZE); // Enum as u8
        layout.add_field("subroutine_type", U8_SIZE); // Enum as u8
        layout.add_field("current_executions", U32_SIZE);
        layout.add_field("bump", U8_SIZE);
        
        // Variable fields
        layout.add_field("label", AccountSizeCalculator::string_size(max_label_length));
        layout.add_field("allowed_users", AccountSizeCalculator::pubkey_vec_size(max_allowed_users));
        
        layout
    }
    
    /// Optimize LibraryInfo account layout
    pub fn optimize_library_info_account(
        max_type_length: usize,
        max_description_length: usize,
        max_version_length: usize,
        max_dependencies: usize,
    ) -> AccountLayout {
        let mut layout = AccountLayout::new("LibraryInfo");
        
        // Fixed fields
        layout.add_field("program_id", PUBKEY_SIZE);
        layout.add_field("is_approved", BOOL_SIZE);
        layout.add_field("last_updated", I64_SIZE);
        layout.add_field("bump", U8_SIZE);
        
        // Variable fields
        layout.add_field("library_type", AccountSizeCalculator::string_size(max_type_length));
        layout.add_field("description", AccountSizeCalculator::string_size(max_description_length));
        layout.add_field("version", AccountSizeCalculator::string_size(max_version_length));
        
        // Dependencies (complex type)
        let dependency_size = PUBKEY_SIZE + // program_id
                             AccountSizeCalculator::string_size(16) + // required_version
                             BOOL_SIZE + // is_optional
                             U8_SIZE; // dependency_type enum
        layout.add_field("dependencies", VEC_HEADER_SIZE + (max_dependencies * dependency_size));
        
        layout
    }
    
    /// Optimize ZK verification key account layout
    pub fn optimize_verification_key_account(
        max_key_data_size: usize,
    ) -> AccountLayout {
        let mut layout = AccountLayout::new("VerificationKey");
        
        layout.add_field("program_id", PUBKEY_SIZE);
        layout.add_field("key_type", U8_SIZE); // Enum as u8
        layout.add_field("is_active", BOOL_SIZE);
        layout.add_field("created_at", I64_SIZE);
        layout.add_field("verification_count", U64_SIZE);
        layout.add_field("bump", U8_SIZE);
        layout.add_field("key_data", AccountSizeCalculator::vec_size::<u8>(max_key_data_size));
        
        layout
    }
}

/// Account layout representation for optimization analysis
#[derive(Debug, Clone)]
pub struct AccountLayout {
    pub name: String,
    pub fields: Vec<FieldLayout>,
    pub total_size: usize,
}

impl AccountLayout {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: Vec::new(),
            total_size: ACCOUNT_HEADER_SIZE, // Start with discriminator
        }
    }
    
    pub fn add_field(&mut self, name: &str, size: usize) {
        self.fields.push(FieldLayout {
            name: name.to_string(),
            size,
            offset: self.total_size,
        });
        self.total_size += size;
    }
    
    pub fn get_rent_cost(&self) -> u64 {
        AccountSizeCalculator::calculate_rent_cost(self.total_size)
    }
    
    pub fn suggest_optimizations(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();
        
        // Check for oversized string fields
        for field in &self.fields {
            if field.name.contains("string") && field.size > 128 {
                suggestions.push(OptimizationSuggestion {
                    field_name: field.name.clone(),
                    suggestion_type: SuggestionType::ReduceStringSize,
                    potential_savings: field.size - 64, // Suggest 64 byte max
                    description: format!("Consider reducing max length for {}", field.name),
                });
            }
        }
        
        // Check for large Vec fields
        for field in &self.fields {
            if field.size > 1000 {
                suggestions.push(OptimizationSuggestion {
                    field_name: field.name.clone(),
                    suggestion_type: SuggestionType::OptimizeVecStorage,
                    potential_savings: field.size / 4, // Estimate 25% savings
                    description: format!("Consider optimizing storage for large field {}", field.name),
                });
            }
        }
        
        suggestions
    }
}

/// Individual field layout information
#[derive(Debug, Clone)]
pub struct FieldLayout {
    pub name: String,
    pub size: usize,
    pub offset: usize,
}

/// Optimization suggestions for account layouts
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub field_name: String,
    pub suggestion_type: SuggestionType,
    pub potential_savings: usize,
    pub description: String,
}

/// Types of optimization suggestions
#[derive(Debug, Clone)]
pub enum SuggestionType {
    ReduceStringSize,
    OptimizeVecStorage,
    PackBooleans,
    OptimizeEnumStorage,
    AddPadding,
    RemoveUnusedFields,
}

/// Macro for easy account size calculation
#[macro_export]
macro_rules! calculate_account_size {
    ($($field:ident: $type:ty),* $(,)?) => {
        {
            let mut size = 8; // Discriminator
            $(
                size += std::mem::size_of::<$type>();
            )*
            size
        }
    };
}

/// Macro for optimized string field sizing
#[macro_export]
macro_rules! optimized_string_size {
    ($max_len:expr) => {
        4 + $max_len // Vec header + max length
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_account_size_calculation() {
        let size = AccountSizeCalculator::total_account_size(100);
        assert_eq!(size, 108); // 8 + 100
    }
    
    #[test]
    fn test_string_size_calculation() {
        let size = AccountSizeCalculator::string_size(32);
        assert_eq!(size, 36); // 4 + 32
    }
    
    #[test]
    fn test_authorization_layout_optimization() {
        let layout = ValenceAccountOptimizer::optimize_authorization_account(32, 10);
        assert!(layout.total_size > 0);
        assert!(layout.fields.len() > 0);
    }
} 