<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('call-to-action', 'gdlr_core_pb_element_call_to_action'); 
	
	if( !class_exists('gdlr_core_pb_element_call_to_action') ){
		class gdlr_core_pb_element_call_to_action{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-bullhorn',
					'title' => esc_html__('Call To Action', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Call To Action Item Title', 'goodlayers-core'),
							),
							'caption' => array(
								'title' => esc_html__('Caption', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Call to cation item caption', 'goodlayers-core'),
							),
							'button-text' => array(
								'title' => esc_html__('Button Text', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Learn More', 'goodlayers-core'),
							),
							'button-link' => array(
								'title' => esc_html__('Button Link', 'goodlayers-core'),
								'type' => 'text',
								'default' => '#'
							),
							'button-link-target' => array(
								'title' => esc_html__('Button Link Target', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'_self' => esc_html__('Current Screen', 'goodlayers-core'),
									'_blank' => esc_html__('New Window', 'goodlayers-core'),
								),
								'default' => '_self'
							),						
						)
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'style' => array(
								'title' => esc_html__('Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'left-align' => GDLR_CORE_URL . '/include/images/call-to-action/left-align.png',
									'center-align' => GDLR_CORE_URL . '/include/images/call-to-action/center-align.png',
									'right-align' => GDLR_CORE_URL . '/include/images/call-to-action/right-align.png',
									'left-align-right-button' => GDLR_CORE_URL . '/include/images/call-to-action/left-align-right-button.png',
									'right-align-left-button' => GDLR_CORE_URL . '/include/images/call-to-action/right-align-left-button.png',
								),
								'wrapper-class' => 'gdlr-core-fullsize',
								'default' => 'center-align'
							),
							'button-style' => array(
								'title' => esc_html__('Button Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'transparent' => esc_html__('Transparent', 'goodlayers-core'),
									'solid' => esc_html__('Solid', 'goodlayers-core'),
									'gradient' => esc_html__('Gradient - Vertical', 'goodlayers-core'),
									'gradient-v' => esc_html__('Gradient - Horizontal', 'goodlayers-core'),
								),
								'default' => 'border'
							),
							'button-border-radius' => array(
								'title' => esc_html__('Button Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'description' => esc_html__('Leave this field blank for default value', 'goodlayers-core'),
							),
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'title-size' => array(
								'title' => esc_html__('Title Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '36px'
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
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'none'
							),
							'caption-size' => array(
								'title' => esc_html__('Caption Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '19px'
							),
							'caption-letter-spacing' => array(
								'title' => esc_html__('Caption Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
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
							),
						)
					),
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-call-to-action-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-call-to-action-<?php echo esc_attr($id); ?>').parent().gdlr_core_content_script();
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'title' => esc_html__('Call To Action Item Title', 'goodlayers-core'),
						'caption' => esc_html__('Call to cation item caption', 'goodlayers-core'),
						'button-text' => esc_html__('Learn More', 'goodlayers-core'),
						'button-link' => '#', 'button-link-target' => '_self', 'button-style' => 'transparent',
						'style' => 'center-align', 'button-style' => '',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// default size
				$settings['style'] = empty($settings['style'])? '': $settings['style'];
				$settings['title-size'] = (empty($settings['title-size']) || $settings['title-size'] == '36px')? '': $settings['title-size'];
				$settings['caption-size'] = (empty($settings['caption-size']) || $settings['caption-size'] == '19px')? '': $settings['caption-size'];
				
				// start printing item
				$extra_class  = 'gdlr-core-style-' . $settings['style'];
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-call-to-action-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				$ret .= '<div class="gdlr-core-call-to-action-item-inner" >';
				$ret .= '<div class="gdlr-core-call-to-action-item-content-wrap">';
				if( !empty($settings['title']) ){
					$ret .= '<h3 class="gdlr-core-call-to-action-item-title" ' . gdlr_core_esc_style(array(
						'font-size'=>$settings['title-size'],
						'font-weight'=>empty($settings['title-font-weight'])? '': $settings['title-font-weight'],
						'letter-spacing'=>empty($settings['title-letter-spacing'])? '': $settings['title-letter-spacing'],
						'text-transform'=>(empty($settings['title-text-transform']) || $settings['title-text-transform'] == 'none')? '': $settings['title-text-transform'],
					)) . ' >' . gdlr_core_text_filter($settings['title']) . '</h3>';
				}
				if( !empty($settings['caption']) ){
					$ret .= '<div class="gdlr-core-call-to-action-item-caption gdlr-core-title-font gdlr-core-skin-caption" ' . gdlr_core_esc_style(array(
						'font-size'=>$settings['caption-size'],
						'letter-spacing'=>empty($settings['caption-letter-spacing'])? '': $settings['caption-letter-spacing'],
					)) . ' >' . gdlr_core_text_filter($settings['caption']) . '</div>';
				}
				$ret .= '</div>'; // gdlr-core-call-to-action-item-wrap
				
				if( !empty($settings['button-text']) && !empty($settings['button-link']) ){
					$ret .= '<div class="gdlr-core-call-to-action-item-button" >';
					$ret .= gdlr_core_get_button(array(
						'button-link' => $settings['button-link'],
						'button-text' => $settings['button-text'],
						'button-link-target' => $settings['button-link-target'],
						'button-background' => empty($settings['button-style'])? 'transparent': $settings['button-style'],
						'border-radius' => empty($settings['button-border-radius'])? '': $settings['button-border-radius']
					));
					$ret .= '</div>';
				}
				$ret .= '</div>'; // gdlr-core-call-to-action-item-inner
				$ret .= '</div>'; // gdlr-core-call-to-action-item
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_content
	} // class_exists	