<?php
	/*
	*	Goodlayers Item For Page Builder
	*/

	gdlr_core_page_builder_element::add_element('price-list', 'gdlr_core_pb_element_price_list');

	if( !class_exists('gdlr_core_pb_element_price_list') ){
		class gdlr_core_pb_element_price_list{

			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-dollar',
					'title' => esc_html__('Price List', 'goodlayers-core')
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
								'title' => esc_html__('Add New Price', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'tabs',
								'wrapper-class' => 'gdlr-core-fullsize',
								'options' => array(
									'icon' => array(
										'title' => esc_html__('Image Icon', 'goodlayers-core'),
										'type' => 'upload'
									),
									'title' => array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'price' => array(
										'title' => esc_html__('Price', 'goodlayers-core'),
										'type' => 'text'
									),
									'caption' => array(
										'title' => esc_html__('Caption', 'goodlayers-core'),
										'type' => 'text'
									),
								),
								'default' => array(
									array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'price' => esc_html__('$2000.00', 'goodlayers-core'),
									),
									array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'price' => esc_html__('$2000.00', 'goodlayers-core'),
									),
									array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'price' => esc_html__('$2000.00', 'goodlayers-core'),
									),
									array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'price' => esc_html__('$2000.00', 'goodlayers-core'),
									),
								)
							),
							'divider-style' => array(
								'title' => esc_html__('Divider Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'' => GDLR_CORE_URL . '/include/images/divider/solid.png',
									'double' => GDLR_CORE_URL . '/include/images/divider/double.png',
									'dotted' => GDLR_CORE_URL . '/include/images/divider/dotted.png',
									'dashed' => GDLR_CORE_URL . '/include/images/divider/dashed.png'
								),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
						),
					),
					'style' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'price-color' => array(
								'title' => esc_html__('Price Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'caption-color' => array(
								'title' => esc_html__('Caption Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'divider-color' => array(
								'title' => esc_html__('Divider Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						),
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'title-size' => array(
								'title' => esc_html__('Title Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'title-font-weight' => array(
								'title' => esc_html__('Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'price-size' => array(
								'title' => esc_html__('Price Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'price-font-weight' => array(
								'title' => esc_html__('Price Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'caption-size' => array(
								'title' => esc_html__('Caption Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'caption-font-weight' => array(
								'title' => esc_html__('Caption Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							)
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'title-top-margin' => array(
								'title' => esc_html__('Title Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'list-bottom-margin' => array(
								'title' => esc_html__('List Bottom Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
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
?><script id="gdlr-core-preview-price-list-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-price-list-<?php echo esc_attr($id); ?>').parent().gdlr_core_tab();
});
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
								'title' => esc_html__('Title', 'goodlayers-core'),
								'price' => esc_html__('$2000.00', 'goodlayers-core'),
							),
							array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'price' => esc_html__('$2000.00', 'goodlayers-core'),
							),
							array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'price' => esc_html__('$2000.00', 'goodlayers-core'),
							),
							array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'price' => esc_html__('$2000.00', 'goodlayers-core'),
							),
						),
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				// start printing item
				$extra_class  = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-price-list-item gdlr-core-item-pdlr gdlr-core-item-pdb clearfix ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( !empty($settings['tabs']) ){
					foreach( $settings['tabs'] as $tab ){
						$ret .= '<div class="gdlr-core-price-list clearfix" ' . gdlr_core_esc_style(array(
							'margin-bottom' => empty($settings['list-bottom-margin'])? '': $settings['list-bottom-margin']
						)) . ' >';
						if( !empty($tab['title']) ){
							$ret .= '<div class="gdlr-core-price-list-title" ' . gdlr_core_esc_style(array(
								'font-size' => empty($settings['title-size'])? '': $settings['title-size'],
								'font-weight' => empty($settings['title-font-weight'])? '': $settings['title-font-weight'], 
								'color' => empty($settings['title-color'])? '': $settings['title-color'],
								'margin-top' => empty($settings['title-top-margin'])? '': $settings['title-top-margin']
							)) . ' >';
							if( !empty($tab['icon']) ){
								$ret .= gdlr_core_get_image($tab['icon']);
							}
							$ret .= gdlr_core_text_filter($tab['title']);
							$ret .= '</div>';
						}
						if( !empty($tab['price']) ){
							$ret .= '<div class="gdlr-core-price-list-price" ' . gdlr_core_esc_style(array(
								'font-size' => empty($settings['price-size'])? '': $settings['price-size'],
								'font-weight' => empty($settings['price-font-weight'])? '': $settings['price-font-weight'], 
								'color' => empty($settings['price-color'])? '': $settings['price-color']
							)) . ' >' . gdlr_core_text_filter($tab['price']) . '</div>';
						}
						$ret .= '<div class="gdlr-core-price-list-divider" ' . gdlr_core_esc_style(array(
							'font-size' => empty($settings['title-size'])? '': $settings['title-size'],
							'border-color' => empty($settings['divider-color'])? '': $settings['divider-color'],
							'border-style' => empty($settings['divider-style'])? '': $settings['divider-style'],
							'padding-top' => empty($settings['title-top-margin'])? '': $settings['title-top-margin']
						)) . ' ></div>';

						if( !empty($tab['caption']) ){
							$ret .= '<div class="clear"></div>';
							$ret .= '<div class="gdlr-core-price-list-caption" ' . gdlr_core_esc_style(array(
								'font-size' => empty($settings['caption-size'])? '': $settings['caption-size'],
								'font-weight' => empty($settings['caption-font-weight'])? '': $settings['caption-font-weight'], 
								'color' => empty($settings['caption-color'])? '': $settings['caption-color']
							)) . ' >' . gdlr_core_text_filter($tab['caption']) . '</div>';
						}
						$ret .= '</div>';
					}
				}

				$ret .= '</div>'; // gdlr-core-price-table-item

				return $ret;
			}

		} // gdlr_core_pb_element_price_table
	} // class_exists
