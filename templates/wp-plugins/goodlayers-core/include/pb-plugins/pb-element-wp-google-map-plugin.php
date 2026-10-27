<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('wp-google-map-plugin', 'gdlr_core_pb_element_wp_google_map_plugin'); 
	
	if( !class_exists('gdlr_core_pb_element_wp_google_map_plugin') ){
		class gdlr_core_pb_element_wp_google_map_plugin{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'icon_map',
					'title' => esc_html__('Wp Google Map Plugin', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'wpgmap-id' => array(
								'title' => esc_html__('Choose Map ( WP GOOGLE MAP PLUGIN )', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => gdlr_core_get_wpgmp_list()
							),
							'wpgmap2-id' => array(
								'title' => esc_html__('Choose Map ( WP GOOGLE MAPS ) ', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => gdlr_core_get_wpgm_list(),
								'description' => esc_html__('This option will be ignored if you select the map from wp-google-map-plugins', 'goodlayers-core')
							)				
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom ( Item )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							)
						)
					)
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);			
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;
				global $wpgmza_tblname_maps;

				// default variable
				if( empty($settings) ){
					$settings = array(
						'wpgmap-id' => '',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-wp-google-map-plugin-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				
				// display
				if( !defined('TBL_MAP') && empty($wpgmza_tblname_maps) ){
					$message = wp_kses(__('Please install and activate the "<a target="_blank" href="https://wordpress.org/plugins/wp-google-map-plugin/" >Wp Google Map Plugin</a>" or "<a target="_blank" href="https://wordpress.org/plugins/wp-google-maps/" >Wp Google Maps</a>" plugin to show the map.', 'goodlayers-core'), 
						array( 'a' => array('target'=>array(), 'href'=>array()) ));
				}else if( empty($settings['wpgmap-id']) && empty($settings['wpgmap2-id']) ){
					$message = esc_html__('Please choose the maps you want to display.', 'goodlayers-core');
				}else if( $preview ){
					if( !empty($settings['wpgmap-id']) ){
						$message = '[put_wpgm id="' . esc_attr($settings['wpgmap-id']) . '"]';
					}else if( !empty($settings['wpgmap2-id']) ){
						$message = '[wpgmza id="' . esc_attr($settings['wpgmap2-id']) . '"]';
					}
				}else{
					if( !empty($settings['wpgmap-id']) ){
						$ret .= do_shortcode('[put_wpgm id="' . esc_attr($settings['wpgmap-id']) . '"]');
					}else if( !empty($settings['wpgmap2-id']) ){
						$ret .= do_shortcode('[wpgmza id="' . esc_attr($settings['wpgmap2-id']) . '"]');
					}
				}
				if( !empty($message) ){
					$ret .= '<div class="gdlr-core-external-plugin-message">' . gdlr_core_escape_content($message) . '</div>';
				}
				
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_wp_google_map_plugin
	} // class_exists	

	// get google map list
	// core/class.map-widget.php
	if( !function_exists('gdlr_core_get_wpgmp_list') ){
		function gdlr_core_get_wpgmp_list(){
			if( !defined('TBL_MAP') ) return array();

			global $wpdb;
			$results = $wpdb->get_results('SELECT map_id, map_title FROM ' . TBL_MAP);

			if( !empty($results) ){
				$maps = array();
				foreach($results as $result){
					if( !empty($result->map_id) && !empty($result->map_title) ){
						$maps[$result->map_id] = $result->map_title;
					}
				}
				return $maps;
			}else{
				return array();
			}
		}
	}

	// wpGoogleMaps.php
	if( !function_exists('gdlr_core_get_wpgm_list') ){
		function gdlr_core_get_wpgm_list(){
		    global $wpdb;
		    global $wpgmza_tblname_maps;
		    
			if( empty($wpgmza_tblname_maps) ){ return array(); }

		    $results = $wpdb->get_results($wpdb->prepare("SELECT * FROM $wpgmza_tblname_maps WHERE 'active' = %d ORDER BY 'id' DESC", 0));

			if( !empty($results) ){
				$maps = array();
				foreach($results as $result){
					if( !empty($result->id) && !empty($result->map_title) ){
						$maps[$result->id] = $result->map_title;
					}
				}
				return $maps;
			}else{
				return array();
			}
		}
	}