SELECT key, value FROM settings WHERE key GLOB ?1 AND key NOT GLOB 'cht_social_[0-9]*_*'
