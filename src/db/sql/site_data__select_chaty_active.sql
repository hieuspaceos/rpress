SELECT 1 FROM settings WHERE value='1' AND (key='cht_active' OR key GLOB 'cht_active_[0-9]*') LIMIT 1
