<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('food_menu', 'gdlr_core_pb_element_food_menu'); 
	
	if( !class_exists('gdlr_core_pb_element_food_menu') ){
		class gdlr_core_pb_element_food_menu{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-folder-o',
					'title' => esc_html__('Food Menu', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'tabs' => array(
								'title' => esc_html__('Add New Tab', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'tabs',
								'wrapper-class' => 'gdlr-core-fullsize',
								'options' => array(
									'title' => array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'price' => array(
										'title' => esc_html__('Price', 'goodlayers-core'),
										'type' => 'text'
									),
									'content' => array(
										'title' => esc_html__('Content', 'goodlayers-core'),
										'type' => 'textarea'
									),
									'featured' => array(
										'title' => esc_html__('Featured', 'goodlayers-core'),
										'type' => 'checkbox',
										'default' => 'disable'
									)
								),
								'default' => array(
									array(
										'title' => esc_html__('Tab Title', 'goodlayers-core'),
										'content' => esc_html__('Tab content area', 'goodlayers-core'),
									),
									array(
										'title' => esc_html__('Tab Title', 'goodlayers-core'),
										'content' => esc_html__('Tab content area', 'goodlayers-core'),
									),
								)
							),
						),
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'title-featured-color' => array(
								'title' => esc_html__('Title Featured Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-color' => array(
								'title' => esc_html__('Content Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'price-color' => array(
								'title' => esc_html__('Price Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'border-color' => array(
								'title' => esc_html__('Border Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						),
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'title-font-size' => array(
								'title' => esc_html__('Title Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'title-font-style' => array(
								'title' => esc_html__('Title Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
								'default' => 'normal'
							),
							'title-font-weight' => array(
								'title' => esc_html__('Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'title-letter-spacing' => array(
								'title' => esc_html__('Title Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'title-text-transform' => array(
								'title' => esc_html__('Title Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => ''
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
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-tab-<?php echo esc_attr($id); ?>" >
if( document.readyState == 'complete' ){
	jQuery(document).ready(function(){
		jQuery('#gdlr-core-preview-tab-<?php echo esc_attr($id); ?>').parent().gdlr_core_tab();
	});
}else{
	jQuery(window).on('load', function(){
		setTimeout(function(){
			jQuery('#gdlr-core-preview-tab-<?php echo esc_attr($id); ?>').parent().gdlr_core_tab();
		}, 300);
	});
}
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}		
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;

				// default variable
				if( empty($settings) ){
					$settings = array(
						'tabs' => array(
							array(
								'title' => esc_html__('Tab Title', 'goodlayers-core'),
								'content' => esc_html__('Tab content area', 'goodlayers-core'),
							),
							array(
								'title' => esc_html__('Tab Title', 'goodlayers-core'),
								'content' => esc_html__('Tab content area', 'goodlayers-core'),
							),
						),
						'align' => 'left',
						'style' => 'style1-horizontal',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				// tab custom style
				$custom_style  = '';
				$custom_style .= empty($settings['title-featured-color'])? '': ' #custom_style_id .gdlr-core-food-menu-list.gdlr-core-featured .gdlr-core-food-menu-title{ color: ' . $settings['title-featured-color'] . '; fill:  ' . $settings['title-featured-color'] . '; }';
				$custom_style .= empty($settings['content-color'])? '': ' #custom_style_id .gdlr-core-food-menu-content{ color: ' . $settings['content-color'] . '; }';
				$custom_style .= empty($settings['price-color'])? '': ' #custom_style_id .gdlr-core-food-menu-price{ color: ' . $settings['price-color'] . '; }';
				$custom_style .= empty($settings['border-color'])? '': ' #custom_style_id .gdlr-core-food-menu-list{ border-color: ' . $settings['border-color'] . '; }';
				$title_style = gdlr_core_esc_style(array(
					'font-size' => empty($settings['title-font-size'])? '': $settings['title-font-size'],
					'font-style' => empty($settings['title-font-style'])? '': $settings['title-font-style'],
					'font-weight' => empty($settings['title-font-weight'])? '': $settings['title-font-weight'],
					'letter-spacing' => empty($settings['title-letter-spacing'])? '': $settings['title-letter-spacing'],
					'text-transform' => empty($settings['title-text-transform'])? '': $settings['title-text-transform'],
					'color' => empty($settings['title-color'])? '': $settings['title-color']
				), false);
				$custom_style .= empty($title_style)? '': ' #custom_style_id .gdlr-core-food-menu-title{ ' . $title_style . '; }';
				
				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_food_menu_id; 

						if( $preview ){
							$gdlr_core_food_menu_id = empty($gdlr_core_food_menu_id)? array(): $gdlr_core_food_menu_id;
							
							// generate unique id so it does not get overwritten in admin area
							$rnd_id = mt_rand(0, 99999);
							while( in_array($rnd_id, $gdlr_core_food_menu_id) ){
								$rnd_id = mt_rand(0, 99999);
							}
							$gdlr_core_food_menu_id[] = $rnd_id;
							$settings['id'] = 'gdlr-core-food-menu-' . $rnd_id;
						}else{
							$gdlr_core_food_menu_id = empty($gdlr_core_food_menu_id)? 1: $gdlr_core_food_menu_id + 1;
							$settings['id'] = 'gdlr-core-food-menu-' . $gdlr_core_food_menu_id;
						}
					}
					$custom_style = str_replace('custom_style_id', $settings['id'], $custom_style); 
					if( $preview ){
						$custom_style = '<style>' . $custom_style . '</style>';
					}else{
						gdlr_core_add_inline_style($custom_style);
						$custom_style = '';
					}
				}
					
				// start printing item
				$ret  = '<div class="gdlr-core-food-menu-item gdlr-core-js gdlr-core-item-pdlr gdlr-core-item-pdb" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				if( !empty($settings['tabs']) ){
					foreach( $settings['tabs'] as $tab ){
						$featured = (!empty($tab['featured']) && $tab['featured'] == 'enable');

						$ret .= '<div class="gdlr-core-food-menu-list ' . ($featured? 'gdlr-core-featured': '') . '" >';
						if( !empty($tab['title']) || !empty($tab['price']) ){
							$ret .= '<div class="gdlr-core-food-menu-title-wrap" >';
							if( !empty($tab['title']) ){
								$ret .= '<h4 class="gdlr-core-food-menu-title" >';
								if( $featured ){
									$ret .= '<svg width="22" height="20" viewBox="0 0 22 20" xmlns="http://www.w3.org/2000/svg"><path d="M7.81128 19.8653L6.02668 16.8576L2.63823 16.1153L2.969 12.6269L0.672852 9.99999L2.969 7.37309L2.63823 3.88464L6.02668 3.14234L7.81128 0.134644L10.9997 1.48849L14.1882 0.134644L15.9728 3.14234L19.3612 3.88464L19.0305 7.37309L21.3266 9.99999L19.0305 12.6269L19.3612 16.1153L15.9728 16.8576L14.1882 19.8653L10.9997 18.5115L7.81128 19.8653ZM8.44973 17.95L10.9997 16.8692L13.5805 17.95L14.9997 15.55L17.7497 14.9192L17.4997 12.1L19.3497 9.99999L17.4997 7.86922L17.7497 5.04999L14.9997 4.44999L13.5497 2.04999L10.9997 3.13077L8.41895 2.04999L6.99973 4.44999L4.24973 5.04999L4.49973 7.86922L2.64973 9.99999L4.49973 12.1L4.24973 14.95L6.99973 15.55L8.44973 17.95ZM9.94973 13.2038L15.2536 7.89999L14.1997 6.81539L9.94973 11.0654L7.79973 8.94617L6.7459 9.99999L9.94973 13.2038Z" /></svg>';
								}
								$ret .= gdlr_core_text_filter($tab['title']);
								$ret .= '</h4>';
							}
							if( !empty($tab['price']) ){
								$ret .= '<div class="gdlr-core-food-menu-price" >' . gdlr_core_text_filter($tab['price']) . '</div>';
							}
							$ret .= '</div>';
						}
						if( !empty($tab['content']) ){
							$ret .= '<div class="gdlr-core-food-menu-content" >' . gdlr_core_content_filter($tab['content']) . '</div>';
						}
						$ret .= '</div>';
					}
				}
				$ret .= '</div>'; // gdlr-core-tab-item
				$ret .= $custom_style;
				
				return $ret;
			}			
			
		} // gdlr_core_pb_element_tab
	} // class_exists	