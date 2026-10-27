<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('type-animation', 'gdlr_core_pb_element_type_animation'); 
	
	if( !class_exists('gdlr_core_pb_element_type_animation') ){
		class gdlr_core_pb_element_type_animation{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-align-justify',
					'title' => esc_html__('Type Animation', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'prefix' => array(
								'title' => esc_html__('Animate Text Prefix', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Text Prefix', 'goodlayers-core'),
							),
							'animate-text-first' => array(
								'title' => esc_html__('Animate Text First', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Animate Text', 'goodlayers-core'),
							),
							'animate-text-second' => array(
								'title' => esc_html__('Animate Text Second', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Second Words', 'goodlayers-core'),
							),
							'suffix' => array(
								'title' => esc_html__('Animate Text Suffix', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Text Suffix', 'goodlayers-core'),
							),		
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'left'
							)
						)
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'border-bottom-width' => array(
								'title' => esc_html__('Border Bottom Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => ''
							),
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'font-size' => array(
								'title' => esc_html__('Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '19px',
								'description' => esc_html__('Leaving this field blank will display the default font size from theme options', 'goodlayers-core'),
							),
							'font-weight' => array(
								'title' => esc_html__('Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'font-style' => array(
								'title' => esc_html__('Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
								'default' => 'normal'
							),
							'font-letter-spacing' => array(
								'title' => esc_html__('Font Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '0px'
							),
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'font-color' => array(
								'title' => esc_html__('Font Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'border-bottom-color' => array(
								'title' => esc_html__('Border Bottom Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'background-color' => array(
								'title' => esc_html__('Background (Highlight) Color', 'goodlayers-core'),
								'type' => 'colorpicker',
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
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-type-animation-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-type-animation-<?php echo esc_attr($id); ?>').parent().gdlr_core_typed_animation();
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
						'prefix' => esc_html__('Text Prefix', 'goodlayers-core'),
						'animate-text-first' => esc_html__('Animate Text', 'goodlayers-core'),
						'animate-text-second' => esc_html__('Second Words', 'goodlayers-core'),
						'suffix' => esc_html__('Text Suffix', 'goodlayers-core'),
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				$animation_text = array();
				if( !empty($settings['animate-text-first']) ){
					$animation_text[] = $settings['animate-text-first'];
				}
				if( !empty($settings['animate-text-second']) ){
					$animation_text[] = $settings['animate-text-second'];
				}
				$settings['text-align'] = empty($settings['text-align'])? 'left': $settings['text-align'];
				$settings['border-bottom-width'] = (empty($settings['border-bottom-width']) || $settings['border-bottom-width'] == '0px')? '': $settings['border-bottom-width'];

				// start printing item
				$extra_class  = ' gdlr-core-' . $settings['text-align'] . '-align';
				$extra_class .= empty($settings['border-bottom-width'])? '': ' gdlr-core-with-border';
				$extra_class .= empty($settings['background-color'])? '': ' gdlr-core-with-highlight';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-type-animation-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				$ret .= '<div class="gdlr-core-type-animation-item-content" ' . gdlr_core_esc_style(array(
					'color' => empty($settings['font-color'])? '': $settings['font-color'],
					'background-color' => empty($settings['background-color'])? '': $settings['background-color'],
					'font-size' => (empty($settings['font-size']) || $settings['font-size'] == '19px')? '': $settings['font-size'],
					'font-weight' => empty($settings['font-weight'])? '': $settings['font-weight'],
					'font-style' => empty($settings['font-style'])? '': $settings['font-style'],
					'letter-spacing' => (empty($settings['font-letter-spacing']) || $settings['font-letter-spacing'] == '0px')? '': $settings['font-letter-spacing'],
					'border-bottom-width' => $settings['border-bottom-width'],
					'border-bottom-color' => empty($settings['border-bottom-color'])? '': $settings['border-bottom-color'],
				)) . ' >';
				if( !empty($settings['prefix']) ){
					$ret .= gdlr_core_text_filter($settings['prefix']) . ' ';
				}	
				if( !empty($animation_text) ){
					$ret .= '<span class="gdlr-core-type-animation-item-animated gdlr-core-js" ';
					$ret .= (sizeof($animation_text) > 1)? 'data-animation-text="' . esc_attr(json_encode($animation_text)) . '"': '';
					$ret .= ' >';
					$ret .= gdlr_core_text_filter($animation_text[0]);
					$ret .= '</span>';
				}
				if( !empty($settings['suffix']) ){
					$ret .= ' ' . gdlr_core_text_filter($settings['suffix']);
				}
				$ret .= '</div>'; // gdlr-core-type-animation-item-content
				$ret .= '</div>'; // gdlr-core-type-animation-item
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_type_animation
	} // class_exists	