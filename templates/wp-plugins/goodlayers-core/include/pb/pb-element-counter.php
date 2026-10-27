<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('counter', 'gdlr_core_pb_element_counter_item'); 
	
	if( !class_exists('gdlr_core_pb_element_counter_item') ){
		class gdlr_core_pb_element_counter_item{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-circle-o-notch',
					'title' => esc_html__('Counter', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'top-text-type' => array(
								'title' => esc_html__('Top Text Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'icon' => esc_html__('Icon', 'goodlayers-core'),
									'text' => esc_html__('Text', 'goodlayers-core'),
								)
							),
							'top-icon' => array(
								'title' => esc_html__('Top Icon', 'goodlayers-core'),
								'type' => 'icons',
								'default' => 'fa fa-cloud',
								'condition' => array(
									'top-text-type' => 'icon'
								),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'top-text' => array(
								'title' => esc_html__('Top Text', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array(
									'top-text-type' => 'text'
								)
							),
							'style' => array(
								'title' => esc_html__('Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'bottom-caption' => esc_html__('Bottom Caption', 'goodlayers-core'),
									'side-caption' => esc_html__('Side Caption', 'goodlayers-core')
								)
							),
							'align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'condition' => array( 'style' => 'bottom-caption' ),
								'default' => 'center'
							),
							'prefix' => array(
								'title' => esc_html__('Prefix', 'goodlayers-core'),
								'type' => 'text'
							),
							'start-number' => array(
								'title' => esc_html__('Start Number', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'default' => 0
							),
							'end-number' => array(
								'title' => esc_html__('End Number', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'default' => 99
							),					
							'animation-time' => array(
								'title' => esc_html__('Number Animation Time', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'default' => 4000,
								'description' => esc_html__('Fill the animation time in milli-second', 'goodlayers-core'),
							),
							'suffix' => array(
								'title' => esc_html__('Suffix', 'goodlayers-core'),
								'type' => 'text',
								'default' => '%'
							),
							'divider' => array(
								'title' => esc_html__('Divider', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array( 'style' => 'bottom-caption' )
							),	
							'bottom-text' => array(
								'title' => esc_html__('Bottom Caption Text', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Counter caption', 'goodlayers-core'),
								'condition' => array( 'style' => 'bottom-caption' )
							),	
							'side-text' => array(
								'title' => esc_html__('Side Caption Text', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Counter caption', 'goodlayers-core'),
								'condition' => array( 'style' => 'side-caption' )
							),	
						),
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'top-icon-size' => array(
								'title' => esc_html__('Top Icon Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '30px'
							),
							'top-text-size' => array(
								'title' => esc_html__('Top Text Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '16px'
							),
							'number-size' => array(
								'title' => esc_html__('Number Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '59px'
							),
							'number-font-weight' => array(
								'title' => esc_html__('Number Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'number-font-style' => array(
								'title' => esc_html__('Number Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
								'default' => 'normal'
							),
							'bottom-text-size' => array(
								'title' => esc_html__('Bottom/Side Caption Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '16px'
							),
							'bottom-text-font-weight' => array(
								'title' => esc_html__('Bottom/Side Caption Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'bottom-text-font-style' => array(
								'title' => esc_html__('Bottom/Side Caption Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
								'default' => 'normal'
							),
							'bottom-text-transform' => array(
								'title' => esc_html__('Bottom/Side Caption Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
								),
								'default' => 'uppercase'
							),
						),
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'title-bottom-margin' => array(
								'title' => esc_html__('Title Bottom Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'divider-bottom-margin' => array(
								'title' => esc_html__('Divider Bottom Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'divider-width' => array(
								'title' => esc_html__('Divider Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'divider-height' => array(
								'title' => esc_html__('Divider Height', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'divider-radius' => array(
								'title' => esc_html__('Divider Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						),
					)
				);
			}

			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-counter-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-counter-<?php echo esc_attr($id); ?>').parent().gdlr_core_counter_item();
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
						'top-text-type' => 'none', 'top-icon' => 'fa fa-cloud', 'top-text' => '', 'prefix' => '', 'start-number' => 0, 'end-number' => 99, 'suffix' => '%', 'divider' => 'enable', 
						'bottom-text' => esc_html__('Counter caption', 'goodlayers-core'),
						'animation-time' => '4000', 
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// default size
				$settings['top-icon-size'] = (empty($settings['top-icon-size']) || $settings['top-icon-size'] == '30px')? '': $settings['top-icon-size'];
				$settings['top-text-size'] = (empty($settings['top-text-size']) || $settings['top-text-size'] == '16px')? '': $settings['top-text-size'];
				$settings['number-size'] = (empty($settings['number-size']) || $settings['number-size'] == '59px')? '': $settings['number-size'];
				$settings['bottom-text-size'] = (empty($settings['bottom-text-size']) || $settings['bottom-text-size'] == '16px')? '': $settings['bottom-text-size'];

				// start printing item
				$extra_class  = empty($settings['class'])? '': $settings['class'];
				if( !empty($settings['style']) && $settings['style'] == 'bottom-caption' ){
					$extra_class .= ' gdlr-core-' . (empty($settings['align'])? 'center': $settings['align']) . '-align';
				}
				$ret  = '<div class="gdlr-core-counter-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				
				// top text
				if( !empty($settings['top-text-type']) ){
					if( $settings['top-text-type'] == 'text' ){
						$ret .= '<div class="gdlr-core-counter-item-top-text gdlr-core-skin-caption" ' . gdlr_core_esc_style(array('font-size'=>$settings['top-text-size'])) . ' >' . gdlr_core_text_filter($settings['top-text']) . '</div>';
					}else if( $settings['top-text-type'] == 'icon' ){
						$ret .= '<div class="gdlr-core-counter-item-top-icon" ' . gdlr_core_esc_style(array('font-size'=>$settings['top-icon-size'])) . ' ><i class="' . esc_attr($settings['top-icon']) . '" ></i></div>';
					}
				}
				
				if( empty($settings['style']) || $settings['style'] == 'bottom-caption' ){

					$ret .= self::get_counter_number($settings);

					// divider
					if( !empty($settings['divider']) && $settings['divider'] == 'enable' ){
						$ret .= '<div class="gdlr-core-counter-item-divider gdlr-core-skin-divider" ' . gdlr_core_esc_style(array(
							'width' => empty($settings['divider-width'])? '': $settings['divider-width'],
							'border-bottom-width' => empty($settings['divider-height'])? '': $settings['divider-height'],
							'border-radius' => empty($settings['divider-radius'])? '': $settings['divider-radius'],
							'margin-bottom'=>empty($settings['divider-bottom-margin'])? '': $settings['divider-bottom-margin']
						)) . '></div>';
					}
					
					// bottom text
					if( !empty($settings['bottom-text']) ){
						$ret .= '<div class="gdlr-core-counter-item-bottom-text gdlr-core-skin-content" ' . gdlr_core_esc_style(array(
							'font-size'=>$settings['bottom-text-size'], 
							'font-weight'=>empty($settings['bottom-text-font-weight'])? '': $settings['bottom-text-font-weight'],
							'font-style'=>empty($settings['bottom-text-font-style'])? '': $settings['bottom-text-font-style'], 
							'text-transform' => (empty($settings['bottom-text-transform']) || $settings['bottom-text-transform'] == 'uppercase')? '': $settings['bottom-text-transform']
						)) . ' >' . gdlr_core_text_filter($settings['bottom-text']) . '</div>';
					}

				}else if( $settings['style'] == 'side-caption' ){

					$ret .= '<div class="gdlr-core-counter-item-side-caption" >';
					$ret .= '<div class="gdlr-core-counter-item-side-caption-left" >';
					$ret .= self::get_counter_number($settings);
					$ret .= '</div>';

					if( !empty($settings['side-text']) ){
						$ret .= '<div class="gdlr-core-counter-item-side-caption-right" ' . gdlr_core_esc_style(array(
							'font-size'=>$settings['bottom-text-size'], 
							'font-weight'=>empty($settings['bottom-text-font-weight'])? '': $settings['bottom-text-font-weight'], 
							'font-style'=>empty($settings['bottom-text-font-style'])? '': $settings['bottom-text-font-style'], 
							'text-transform' => (empty($settings['bottom-text-transform']) || $settings['bottom-text-transform'] == 'uppercase')? '': $settings['bottom-text-transform']
						)) . ' >';
						$ret .= gdlr_core_text_filter($settings['side-text']);
						$ret .= '</div>';
					}
					$ret .= '</div>';
				}
				$ret .= '</div>'; // gdlr-core-counter-item
				
				return $ret;
			}

			// Counter number
			static function get_counter_number($settings){

				$ret  = '<div class="gdlr-core-counter-item-number gdlr-core-skin-title gdlr-core-title-font" ' . gdlr_core_esc_style(array(
					'font-size'=>$settings['number-size'],
					'font-weight'=>empty($settings['number-font-weight'])? '': $settings['number-font-weight'],
					'font-style' => empty($settings['number-font-style'])? '': $settings['number-font-style'],
					'margin-bottom'=>empty($settings['title-bottom-margin'])? '': $settings['title-bottom-margin']
				)) . ' >';
				if( !empty($settings['prefix']) ){
					$ret .= '<span class="gdlr-core-counter-item-prefix">' . gdlr_core_text_filter($settings['prefix']) . '</span>';
				}
				if( isset($settings['end-number']) ){
					$ret .= '<span class="gdlr-core-counter-item-count gdlr-core-js" ';
					$ret .= isset($settings['animation-time'])? 'data-duration="' . esc_attr($settings['animation-time']) . '" ': '';
					$ret .= isset($settings['start-number'])? 'data-counter-start="' . esc_attr($settings['start-number']) . '" ': '';
					$ret .= isset($settings['end-number'])? 'data-counter-end="' . esc_attr($settings['end-number']) . '" ': '';
					$ret .= '>' . gdlr_core_escape_content($settings['start-number']) . '</span>';
				}
				if( !empty($settings['suffix']) ){
					$ret .= '<span class="gdlr-core-counter-item-suffix">' . gdlr_core_escape_content($settings['suffix']) . '</span>';
				}
				$ret .= '</div>'; // gdlr-core-counter-item-number

				return $ret;
			}
			
		} // gdlr_core_pb_element_column_service
	} // class_exists	