<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('price-plan', 'gdlr_core_pb_element_price_plan'); 
	
	if( !class_exists('gdlr_core_pb_element_price_plan') ){
		class gdlr_core_pb_element_price_plan{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-folder-o',
					'title' => esc_html__('Price Plan CF7', 'goodlayers-core')
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
									'content' => array(
										'title' => esc_html__('Content', 'goodlayers-core'),
										'type' => 'textarea'
									),
									'price' => array(
										'title' => esc_html__('Price Text', 'goodlayers-core'),
										'type' => 'text'
									),
									'form' => array(
										'title' => esc_html__('Form Shortcode', 'goodlayers-core'),
										'type' => 'textarea'
									)
								),
								'default' => array(
									array(
										'title' => esc_html__('One Year Plan', 'goodlayers-core'),
										'content' => esc_html__('Plan Content Here', 'goodlayers-core'),
									),
									array(
										'title' => esc_html__('One Year Plan', 'goodlayers-core'),
										'content' => esc_html__('Plan Content Here', 'goodlayers-core'),
									),
								)
							),
						),
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'button-skewx' => array(
								'title' => esc_html__('Button Skew X', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'description' => esc_html__('Only input number here', 'goodlayers-core')
							)
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'background-color' => array(
								'title' => esc_html__('Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-color' => array(
								'title' => esc_html__('Content Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'active-color' => array(
								'title' => esc_html__('Active Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							)
						),
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
				
				ob_start();
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
								'title' => esc_html__('One Year Plan', 'goodlayers-core'),
								'content' => esc_html__('Plan Content Here', 'goodlayers-core'),
							),
							array(
								'title' => esc_html__('One Year Plan', 'goodlayers-core'),
								'content' => esc_html__('Plan Content Here', 'goodlayers-core'),
							),
						),
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				// tab custom style
				$custom_style  = '';

				if( !empty($settings['active-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-price-plan.gdlr-core-active .gdlr-core-price-plan-option span{ background: ' . $settings['active-color'] . '; }';
					$custom_style .= '#custom_style_id.gdlr-core-price-plan-item .gdlr-core-price-plan.gdlr-core-active{ border-color: ' . $settings['active-color'] . '; }';
				}

				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_price_plan_id; 

						if( $preview ){
							$gdlr_core_price_plan_id = empty($gdlr_core_price_plan_id)? array(): $gdlr_core_price_plan_id;
							
							// generate unique id so it does not get overwritten in admin area
							$rnd_price_plan_id = mt_rand(0, 99999);
							while( in_array($rnd_price_plan_id, $gdlr_core_price_plan_id) ){
								$rnd_price_plan_id = mt_rand(0, 99999);
							}
							$gdlr_core_price_plan_id[] = $rnd_price_plan_id;
							$settings['id'] = 'gdlr-core-price-plan-' . $rnd_price_plan_id;
						}else{
							$gdlr_core_price_plan_id = empty($gdlr_core_price_plan_id)? 1: $gdlr_core_price_plan_id + 1;
							$settings['id'] = 'gdlr-core-price-plan-' . $gdlr_core_price_plan_id;
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
				$ret  = '<div class="gdlr-core-price-plan-item gdlr-core-js gdlr-core-item-pdlr gdlr-core-item-pdb" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				// price plan options
				$ret .= '<div class="gdlr-core-price-plan-options" >';
				if( !empty($settings['tabs']) ){
					$count = 0;
					foreach( $settings['tabs'] as $tab ){ $count++;
						$ret .= '<div class="gdlr-core-price-plan clearfix ' . ($count == 1? 'gdlr-core-active': '') . '" data-tab-id="' . esc_attr($count) . '" ' . gdlr_core_esc_style(array(
							'background' => empty($settings['background-color'])? '': $settings['background-color']
						)) . ' >';
						$ret .= '<div class="gdlr-core-price-plan-option" ><span></span></div>';
						if( !empty($tab['title']) ){
							$ret .= '<div class="gdlr-core-price-plan-title" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['title-color'])? '': $settings['title-color']
							)) . ' >' . gdlr_core_text_filter($tab['title']) . '</div>';
						}
						if( !empty($tab['price']) ){
							$ret .= '<div class="gdlr-core-price-plan-price" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['title-color'])? '': $settings['title-color']
							)) . ' >' . gdlr_core_text_filter($tab['price']) . '</div>';
						}
						if( !empty($tab['content']) ){
							$ret .= '<div class="gdlr-core-price-plan-content" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['content-color'])? '': $settings['content-color']
							)) . ' >' . gdlr_core_text_filter($tab['content']) . '</div>';
						}	
						$ret .= '</div>'; // gdlr-core-price-plan
					}
				}

				$skewx = empty($settings['button-skewx'])? '': intval($settings['button-skewx']);
				$ret .= '<div class="gdlr-core-step2 gdlr-core-button gdlr-core-rectangle" ' . gdlr_core_esc_style(array(
					'skewx' => $skewx
				)) . ' ><span class="gdlr-core-content" ' . gdlr_core_esc_style(array(
					'skewx' => intval($skewx) * -1
				)) . ' >' . esc_html__('Next Step', 'goodlayers-core') . '<i class="gdlr-icon-oblique-arrow" ></i></span></div>';
				$ret .= '</div>';
				
				// price plan forms
				$ret .= '<div class="gdlr-core-price-plan-forms" >';
				if( !empty($settings['tabs']) ){
					foreach( $settings['tabs'] as $tab ){ 
						$ret .= '<div>';
						$ret .= '<div class="gdlr-core-price-plan-selected-title" >' . esc_html__('You select', 'goodlayers-core') . ' : <strong>' . gdlr_core_text_filter($tab['title']) . '</strong></div>';
						if( !empty($tab['form']) ){
							$ret .= gdlr_core_content_filter($tab['form']);
						}
						$ret .= '<div class="gdlr-core-price-plan-selected-back gdlr-core-step1" ><i class="ion-ios-arrow-thin-left"></i>' . esc_html__('Go back to previous step', 'goodlayers-core') . '</div>';
						$ret .= '</div>';
					}
				}
				$ret .= '</div>';

				$ret .= '</div>'; // gdlr-core-tab-item
				$ret .= $custom_style;
				
				return $ret;
			}			
			
		} // gdlr_core_pb_element_price_plan
	} // class_exists	
