<?php
	/*	
	*	Goodlayers Item To Register Skin
	*/

	if( !class_exists('gdlr_core_skin_settings') ){
		class gdlr_core_skin_settings{

			// deprecate since 1.0.9
			// get skin option
			static function register_skin_option( $skin_option ){}
			// register css for custom style writing
			static function register_selector( $skin, $selector ){}

			// get skin value
			static function get_option(){
				$skin_slug = apply_filters('gdlr_core_skin_option_slug', array());

				$skin_val = array();
				if( !empty($skin_slug) ){
					if( is_array($skin_slug) ){
						if( !empty($skin_slug[0]) ){
							$skin_parent = get_option($skin_slug[0], array());
							$skin_val = empty($skin_parent[$skin_slug[1]])? array(): $skin_parent[$skin_slug[1]];
						}
					}else{
						$skin_val = get_option($skin_slug, array());
					}
				}

				return $skin_val;
			}

			// update skin value
			static function update_option( $new_value ){
				$skin_slug = apply_filters('gdlr_core_skin_option_slug', array());

				if( !empty($skin_slug) ){
					if( is_array($skin_slug) ){
						$skin_parent = get_option($skin_slug[0], array());
						$skin_parent[$skin_slug[1]] = $new_value;

						update_option($skin_slug[0], $skin_parent);
					}else{	
						update_option($skin_slug, $new_value);
					}
				}

				do_action('gdlr_core_theme_option_filewrite');
			}
			
			// get skin for displaying in option
			static function get_skins(){

				$ret = array(
					'' => esc_html__('None', 'goodlayers-core')
				);

				$skins = self::get_option();
				foreach( $skins as $skin ){
					if( !empty($skin['name']) ){
						$ret[$skin['name']] = $skin['name'];
					}
				}

				return $ret;
			}
			
			// get skin css for custom style writing
			static function get_skins_style(){
				
				$ret = '';

				$skins = self::get_option();
				$skin_options = apply_filters('gdlr_core_skin_options', array());

				if( !empty($skins) ){
					foreach( $skins as $skin ){
						foreach( $skin_options as $slug => $skin_option ){
							
							if( !empty($skin['name']) && !empty($skin[$slug]) && !empty($skin_option['selector']) ){
								$selector = str_replace('#gdlr-core-skin#', '.gdlr-core-page-builder-body [data-skin="' . $skin['name'] . '"]', $skin_option['selector']);
								// $selector = str_replace('#gdlr-core-skin#', '.gdlr-core-page-builder-body [data-skin="' . $skin['name'] . '"]', $selector) . "\n" .
								// 			   str_replace('#gdlr-core-skin#', '.gdlr-core-page-builder-body .gdlr-core-pbf-column[data-skin="' . $skin['name'] . '"]', $selector);
								$selector = str_replace('#gdlr#', $skin[$slug], $selector);
								$ret .= $selector . "\n";
							}
							
						}
					}
				}

				return apply_filters('gdlr_core_skin_css', $ret, $skins);
			}

		} // gdlr_core_skin_settings
	} // class_exists