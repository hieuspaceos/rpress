<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('custom-menu', 'gdlr_core_pb_element_custom_menu'); 
	
	if( !class_exists('gdlr_core_pb_element_custom_menu') ){
		class gdlr_core_pb_element_custom_menu{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-list',
					'title' => esc_html__('Custom Menu', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				$menus = wp_get_nav_menus();
				$menu_options = array();
				foreach( $menus as $menu ){
					$menu_options[$menu->term_id] = $menu->name;
				}

				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(	
							'menu' => array(
								'title' => esc_html__('Select Menu', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => $menu_options,
								'description' => esc_html__('This item will only display top level menu (depth = 1)', 'goodlayers-core')
							),
							'style' => array(
								'title' => esc_html__('Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'style-1' => esc_html__('Style 1', 'goodlayers-core'),
									'style-2' => esc_html__('Style 2', 'goodlayers-core'),
								)
							)
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'font-size' => array(
								'title' => esc_html__('Font Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '16px'
							),
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
				$content  = self::get_content($settings);
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				$extra_class = empty($settings['style'])? ' gdlr-core-style-1': ' gdlr-core-' . $settings['style'];
				
				// start printing item
				$ret  = '<div class="gdlr-core-custom-menu-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array(
						'padding-bottom' => $settings['padding-bottom'],
						'font-size' => empty($settings['font-size'])? '': $settings['font-size']
					));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				
				if( !empty($settings['menu']) ){
					$ret .= wp_nav_menu(array(
						'container' => false,
						'menu' => $settings['menu'],
						'echo' => false,
						'depth' => 1
					));
				}else{
					$ret .= '<div class="gdlr-core-external-plugin-message">' . esc_html__('Please select the menu you want to show', 'goodlayers-core') . '</div>';
				}
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_custom_menu
	} // class_exists	