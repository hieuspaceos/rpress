-- Literal prefix match via instr (kept `_like` filename — churn over accuracy).
SELECT option_name, option_value FROM wp_options WHERE instr(option_name, ?1) = 1
