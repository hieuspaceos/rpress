<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('tab', 'gdlr_core_pb_element_tab'); 
	
	if( !class_exists('gdlr_core_pb_element_tab') ){
		class gdlr_core_pb_element_tab{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-folder-o',
					'title' => esc_html__('Tab', 'goodlayers-core')
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
										'type' => 'tmce'
									),
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
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'style' => array(
								'title' => esc_html__('Tab Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'style1-horizontal' => GDLR_CORE_URL . '/include/images/tab/style1-horizontal.png',
									'style1-vertical' => GDLR_CORE_URL . '/include/images/tab/style1-vertical.png',
									'style2-horizontal' => GDLR_CORE_URL . '/include/images/tab/style2-horizontal.png',
									'style2-vertical' => GDLR_CORE_URL . '/include/images/tab/style2-vertical.png',
									'style3-horizontal' => GDLR_CORE_URL . '/include/images/tab/style3-horizontal.jpg',
								),
								'default' => 'style1-horizontal',
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'align' => array(
								'title' => esc_html__('Tab Alignment', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'left',
								'description' => esc_html__('Center style only supports with horizontal tab', 'goodlayers-core')
							)
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'tab-title-color' => array(
								'title' => esc_html__('Tab Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'tab-title-active-color' => array(
								'title' => esc_html__('Tab Title Active Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'tab-title-background-color' => array(
								'title' => esc_html__('Tab Title Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'tab-title-active-background-color' => array(
								'title' => esc_html__('Tab Title Active Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'tab-title-border-color' => array(
								'title' => esc_html__('Tab Title Border Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'tab-title-border-active-color' => array(
								'title' => esc_html__('Tab Title Border Active Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'tab-content-color' => array(
								'title' => esc_html__('Tab Content Color', 'goodlayers-core'),
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
									'normal' => esc_html__('Normal', 'goodlayers-core'),
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
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'uppercase'
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
				
				$settings['style'] = empty($settings['style'])? 'style1-horizontal': $settings['style'];
				$settings['align'] = empty($settings['align'])? 'left': $settings['align'];
				if( in_array($settings['style'], array('style1-vertical', 'style2-vertical')) && $settings['align'] == 'center' ){
					$settings['align'] = 'left';
				}

				$tab_item_class  = ' gdlr-core-' . esc_attr($settings['align']) . '-align';
				$tab_item_class .= ' gdlr-core-tab-' . esc_attr($settings['style']);
				$tab_item_class .= empty($settings['no-pdlr'])? ' gdlr-core-item-pdlr': '';
				$tab_item_class .= empty($settings['class'])? '': ' ' . $settings['class'];

				// tab custom style
				$custom_style  = '';
				$custom_style .= empty($settings['tab-title-color'])? '': ' #custom_style_id .gdlr-core-tab-item-title{ color: ' . $settings['tab-title-color'] . '; }';
				$custom_style .= empty($settings['tab-title-active-color'])? '': ' #custom_style_id .gdlr-core-tab-item-title.gdlr-core-active{ color: ' . $settings['tab-title-active-color'] . '; }';
				$custom_style .= empty($settings['tab-title-background-color'])? '': ' #custom_style_id.gdlr-core-tab-style1-horizontal .gdlr-core-tab-item-title, #custom_style_id.gdlr-core-tab-style1-vertical .gdlr-core-tab-item-title{ background-color: ' . $settings['tab-title-background-color'] . '; }';
				$custom_style .= empty($settings['tab-title-active-background-color'])? '': ' #custom_style_id.gdlr-core-tab-style1-horizontal .gdlr-core-tab-item-title.gdlr-core-active, #custom_style_id.gdlr-core-tab-style1-vertical .gdlr-core-tab-item-title.gdlr-core-active{ background-color: ' . $settings['tab-title-active-background-color'] . '; }';
				$custom_style .= empty($settings['tab-title-border-color'])? '': ' #custom_style_id .gdlr-core-tab-item-title-wrap, #custom_style_id .gdlr-core-tab-item-content-wrap, #custom_style_id .gdlr-core-tab-item-title{ border-color: ' . $settings['tab-title-border-color'] . '; }';
				$custom_style .= empty($settings['tab-title-border-active-color'])? '': ' #custom_style_id .gdlr-core-tab-item-title-line{ border-color: ' . $settings['tab-title-border-active-color'] . '; color: ' . $settings['tab-title-border-active-color'] . '; }';
				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_tab_id; 

						if( $preview ){
							$gdlr_core_tab_id = empty($gdlr_core_tab_id)? array(): $gdlr_core_tab_id;
							
							// generate unique id so it does not get overwritten in admin area
							$rnd_tab_id = mt_rand(0, 99999);
							while( in_array($rnd_tab_id, $gdlr_core_tab_id) ){
								$rnd_tab_id = mt_rand(0, 99999);
							}
							$gdlr_core_tab_id[] = $rnd_tab_id;
							$settings['id'] = 'gdlr-core-tab-' . $rnd_tab_id;
						}else{
							$gdlr_core_tab_id = empty($gdlr_core_tab_id)? 1: $gdlr_core_tab_id + 1;
							$settings['id'] = 'gdlr-core-tab-' . $gdlr_core_tab_id;
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

				$tab_title_css = array(
					'font-size' => empty($settings['title-font-size'])? '': $settings['title-font-size'],
					'font-style' => empty($settings['title-font-style'])? '': $settings['title-font-style'],
					'font-weight' => empty($settings['title-font-weight'])? '': $settings['title-font-weight'],
					'letter-spacing' => empty($settings['title-letter-spacing'])? '': $settings['title-letter-spacing'],
					'text-transform' => empty($settings['title-text-transform'])? '': $settings['title-text-transform'],
				);
					
				// start printing item
				$ret  = '<div class="gdlr-core-tab-item gdlr-core-js gdlr-core-item-pdb ' . esc_attr($tab_item_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				if( !empty($settings['tabs']) ){
					$count = 0; $active = 1;
					$ret .= '<div class="gdlr-core-tab-item-title-wrap clearfix gdlr-core-title-font" >';
					foreach( $settings['tabs'] as $tab ){ $count++;
						$ret .= '<div class="gdlr-core-tab-item-title ' . ($count == $active? ' gdlr-core-active': '') . '" data-tab-id="' . esc_attr($count) . '" ' . gdlr_core_esc_style($tab_title_css) . ' >' . gdlr_core_text_filter($tab['title']) . '</div>';
					}
					if( in_array($settings['style'], array('style2-vertical', 'style2-horizontal', 'style3-horizontal')) ){
						$ret .= '<div class="gdlr-core-tab-item-title-line gdlr-core-skin-divider"></div>';
					}
					$ret .= '</div>'; // gdlr-core-tab-item-tab-title-wrap
					
					$count = 0;
					$ret .= '<div class="gdlr-core-tab-item-content-wrap clearfix" >';
					foreach( $settings['tabs'] as $tab ){ $count++;
						$ret .= '<div class="gdlr-core-tab-item-content ' . ($count == $active? ' gdlr-core-active': '') . '" data-tab-id="' . esc_attr($count) . '" ' . gdlr_core_esc_style(array(
							'color'=>empty($settings['tab-content-color'])? '': $settings['tab-content-color']
						)) . ' >' . gdlr_core_content_filter($tab['content']) . '</div>';
					}
					$ret .= '</div>'; // gdlr-core-tab-item-tab
				}
				$ret .= '</div>'; // gdlr-core-tab-item
				$ret .= $custom_style;
				
				return $ret;
			}			
			
		} // gdlr_core_pb_element_tab
	} // class_exists	

	// [gdlr_core_tabs]
	// [gdlr_core_tab title="title 1"]Tab 1[/gdlr_core_tab]
	// [gdlr_core_tab title="title 2"]Tab 2[/gdlr_core_tab]
	// [gdlr_core_tab title="title 3"]Tab 3[/gdlr_core_tab]
	// [/gdlr_core_tabs]
	add_shortcode('gdlr_core_tabs', 'gdlr_core_tabs_shortcode');
	if( !function_exists('gdlr_core_tabs_shortcode') ){
		function gdlr_core_tabs_shortcode($atts, $content = ''){
			$atts = wp_parse_args($atts, array(
				'no-pdlr' => true
			));

			global $gdlr_core_tabs;
			$gdlr_core_tabs = array();

			do_shortcode($content);
			$atts['tabs'] = $gdlr_core_tabs;

			return gdlr_core_pb_element_tab::get_content($atts);
		}
	}	

	add_shortcode('gdlr_core_tab', 'gdlr_core_tab_shortcode');
	if( !function_exists('gdlr_core_tab_shortcode') ){
		function gdlr_core_tab_shortcode($atts, $content = ''){
			$atts = wp_parse_args($atts, array());

			global $gdlr_core_tabs;

			$atts['content'] = gdlr_core_text_filter($content);
			$gdlr_core_tabs[] = $atts;
		}
	}	