<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('skill-circle', 'gdlr_core_pb_element_skill_circle_item'); 
	
	if( !class_exists('gdlr_core_pb_element_skill_circle_item') ){
		class gdlr_core_pb_element_skill_circle_item{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'icon_piechart',
					'title' => esc_html__('Skill Circle', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'item-size' => array(
								'title' => esc_html__('Item Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'small' => esc_html__('Small', 'goodlayers-core'),
									'large' => esc_html__('Large', 'goodlayers-core'),
									'custom' => esc_html__('Custom', 'goodlayers-core'),
								),
								'default' => 'large'
							),
							'item-width' => array(
								'title' => esc_html__('Item Width (PX)', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '200px',
								'condition' => array( 'item-size' => 'custom' ),
								'description' => esc_html__('Specify the initial item width, the item will still resize to fit the container though.', 'goodlayers-core')
							),
							'line-width' => array(
								'title' => esc_html__('Line Width (Without PX)', 'goodlayers-core'),
								'type' => 'text',
								'default' => 10,
								'condition' => array( 'item-size' => 'custom' )
							),
							'heading-text' => array(
								'title' => esc_html__('Heading Text', 'goodlayers-core'),
								'type' => 'text'
							), 
							'percent' => array(
								'title' => esc_html__('Percent', 'goodlayers-core'),
								'type' => 'text',
								'default' => 80,
								'data-input-type' => 'number',
								'description' => esc_html__('Only fill the number here', 'goodlayers-core'),
							), 
							'bar-text' => array(
								'title' => esc_html__('Bar Text', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Skill Circle', 'goodlayers-core'),
							), 
						),
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'item-align' => array(
								'title' => esc_html__('Item Alignment', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center'
							),
							'skill-circle-width' => array(
								'title' => esc_html__('Skill Circle Max Width (Percent)', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Only fill a number here. This option will be applied if side description is not empty.', 'goodlayers-core'),
								'condition' => array( 'item-align' => array('left', 'right') )
							),
							'side-description-title' => array(
								'title' => esc_html__('Side Description Title', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'item-align' => array('left', 'right') )
							),
							'side-description-text' => array(
								'title' => esc_html__('Side Description Text', 'goodlayers-core'),
								'type' => 'textarea',
								'condition' => array( 'item-align' => array('left', 'right') )
							)
						)
					),					
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'heading-text-color' => array(
								'title' => esc_html__('Heading Text Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'default' => '#2d2d2d'
							),
							'text-color' => array(
								'title' => esc_html__('Text Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'default' => '#2d2d2d'
							),
							'bar-filled-color' => array(
								'title' => esc_html__('Bar Filled Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'default' => '#2d9bea'
							),
							'bar-background-color' => array(
								'title' => esc_html__('Bar Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'default' => '#dcdcdc'
							),
							'circle-background-color' => array(
								'title' => esc_html__('Circle Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'side-description-title-color' => array(
								'title' => esc_html__('Side Description Title Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'side-description-text-color' => array(
								'title' => esc_html__('Side Description Text Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'heading-text-font-size' => array(
								'title' => esc_html__('Heading Text Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'heading-text-font-weight' => array(
								'title' => esc_html__('Heading Text Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'bar-text-font-size' => array(
								'title' => esc_html__('Bar Text Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'bar-text-font-weight' => array(
								'title' => esc_html__('Bar Text Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'side-description-title-font-size' => array(
								'title' => esc_html__('Side Description Title Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'side-description-title-font-weight' => array(
								'title' => esc_html__('Side Description Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'side-description-text-font-size' => array(
								'title' => esc_html__('Side Description Text Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'side-description-text-font-weight' => array(
								'title' => esc_html__('Side Description Text Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'side-description-title-top-margin' => array(
								'title' => esc_html__('Side Description Title Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'side-description-title-bottom-margin' => array(
								'title' => esc_html__('Side Description Title Bottom Margin', 'goodlayers-core'),
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
				$content  = self::get_content($settings);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-skill-circle-<?php echo esc_attr($id); ?>" >
if( document.readyState == 'complete' ){
	jQuery(document).ready(function(){
		jQuery('#gdlr-core-preview-skill-circle-<?php echo esc_attr($id); ?>').parent().gdlr_core_skill_circle();
	});
}else{
	jQuery(window).on('load', function(){
		jQuery('#gdlr-core-preview-skill-circle-<?php echo esc_attr($id); ?>').parent().gdlr_core_skill_circle();
	});
}
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
						'item-size' => 'large', 'percent' => 80, 'heading-text' => '', 'item-align' => 'center',
						'bar-text'=>esc_html__('Skill Circle', 'goodlayers-core'),
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				$extra_class  = empty($settings['class'])? '': $settings['class'];

				// default size
				$settings['top-icon-size'] = (empty($settings['top-icon-size']) || $settings['top-icon-size'] == '30px')? '': $settings['top-icon-size'];
				$settings['top-text-size'] = (empty($settings['top-text-size']) || $settings['top-text-size'] == '16px')? '': $settings['top-text-size'];
				$settings['number-size'] = (empty($settings['number-size']) || $settings['number-size'] == '59px')? '': $settings['number-size'];
				$settings['bottom-text-size'] = (empty($settings['bottom-text-size']) || $settings['bottom-text-size'] == '16px')? '': $settings['bottom-text-size'];
				$settings['item-align'] = empty($settings['item-align'])? 'center': $settings['item-align'];
				$settings['item-size'] = empty($settings['item-size'])? 'large': $settings['item-size'];
				
				if( $settings['item-size'] == 'small' ){
					$item_size = '165px';
					$line_width = 4;
				}else if( $settings['item-size'] == 'large' ){
					$item_size = '265px';
					$line_width = 6;
				}else{
					$item_size = empty($settings['item-width'])? '265px': $settings['item-width'];
					$line_width = empty($settings['line-width'])? '6': $settings['line-width'];
				}

				// start printing item
				$ret  = '<div class="gdlr-core-skill-circle-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . ' clearfix" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				
				if( in_array($settings['item-align'], array('left', 'right')) ){
					if( !empty($settings['side-description-text']) || !empty($settings['side-description-title']) ){
						$ret .= '<div class="gdlr-core-skill-circle-main-content-wrap" >';
						$ret .= '<div class="gdlr-core-skill-circle-main-content gdlr-core-' . esc_attr($settings['item-align']) . '" ';
						if( !empty($settings['skill-circle-width']) && is_numeric($settings['skill-circle-width']) ){
							$ret .= gdlr_core_esc_style(array(
								'width' => $settings['skill-circle-width'] . '%'
							));
						}
						$ret .= ' >';
					}	
				}

				$item_class  = ' gdlr-core-skill-circle gdlr-core-js';
				$item_class .= ' gdlr-core-skill-circle-align-' . esc_attr($settings['item-align']);
				$item_class .= ' gdlr-core-skill-circle-size-' . esc_attr($settings['item-size']);
				
				$ret .= '<div class="' . esc_attr($item_class) . '" ';
				$ret .= 'data-percent="' . esc_attr($settings['percent']) . '" ';
				$ret .= 'data-width="' . esc_attr($item_size) . '" ';
				$ret .= 'data-duration="1200" ';
				$ret .= 'data-line-width="' . esc_attr($line_width) . '" ';
				$ret .= 'data-filled-color="' . (empty($settings['bar-filled-color'])? '#2d9bea': esc_attr($settings['bar-filled-color'])) . '" '; 
				$ret .= 'data-filled-background="' . (empty($settings['bar-background-color'])? '#dcdcdc': esc_attr($settings['bar-background-color'])) . '" '; 
				$ret .= gdlr_core_esc_style(array(
					'width' => $item_size, 
					'height' => $item_size,
					'border-color' => empty($settings['bar-background-color'])? '': $settings['bar-background-color'],	
					'background-color' => empty($settings['circle-background-color'])? '': $settings['circle-background-color']
				)) . ' >'; 
				$ret .= '<div class="gdlr-core-skill-circle-content gdlr-core-title-font" >';
				$ret .= '<div class="gdlr-core-skill-circle-head" ' . gdlr_core_esc_style(array(
					'color' => empty($settings['heading-text-color'])? '#2d2d2d': $settings['heading-text-color'],
					'font-size' => empty($settings['heading-text-font-size'])? '': $settings['heading-text-font-size'],
					'font-weight' => empty($settings['heading-text-font-weight'])? '': $settings['heading-text-font-weight'],
				)) . ' >';
				if( !empty($settings['heading-text']) ){
					$ret .= gdlr_core_escape_content($settings['heading-text']);
				}else{
					$ret .= $settings['percent'] . '%';
				}
				$ret .= '</div>'; // gdlr-core-skill-circle-head
				
				if( !empty($settings['bar-text']) ){
					$ret .= '<div class="gdlr-core-skill-circle-caption" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['text-color'])? '#2d2d2d': $settings['text-color'],
						'font-size' => empty($settings['bar-text-font-size'])? '': $settings['bar-text-font-size'],
						'font-weight' => empty($settings['bar-text-font-weight'])? '': $settings['bar-text-font-weight'],
					)) . ' >';
					$ret .= gdlr_core_escape_content($settings['bar-text']);
					$ret .= '</div>';
				}
				$ret .= '</div>'; // gdlr-core-skill-circle-content
				$ret .= '</div>'; // gdlr-core-skill-circle

				if( in_array($settings['item-align'], array('left', 'right')) ){
					if( !empty($settings['side-description-text']) || !empty($settings['side-description-title']) ){
						$ret .= '</div>'; // gdlr-core-skill-circle-main-content

						$ret .= '<div class="gdlr-core-skill-circle-side-description" >';
						if( !empty($settings['side-description-title']) ){
							$ret .= '<div class="gdlr-core-skill-circle-side-description-title gdlr-core-skin-title" ' . gdlr_core_esc_style(array(
								'font-size' => empty($settings['side-description-title-font-size'])? '': $settings['side-description-title-font-size'],
								'font-weight' => empty($settings['side-description-title-font-weight'])? '': $settings['side-description-title-font-weight'],
								'margin-top' => empty($settings['side-description-title-top-margin'])? '': $settings['side-description-title-top-margin'],
								'margin-bottom' => empty($settings['side-description-title-bottom-margin'])? '': $settings['side-description-title-bottom-margin'],
								'color' => empty($settings['side-description-title-color'])? '': $settings['side-description-title-color']
							)) .' >' . gdlr_core_text_filter($settings['side-description-title']) . '</div>';
						}
						if( !empty($settings['side-description-text']) ){
							$ret .= '<div class="gdlr-core-skill-circle-side-description-text" ' . gdlr_core_esc_style(array(
								'font-size' => empty($settings['side-description-text-font-size'])? '': $settings['side-description-text-font-size'],
								'font-weight' => empty($settings['side-description-text-font-weight'])? '': $settings['side-description-text-font-weight'],
								'color' => empty($settings['side-description-text-color'])? '': $settings['side-description-text-color'],
							)) .' >' . gdlr_core_text_filter($settings['side-description-text']) . '</div>';
						}
						$ret .= '</div>';
						$ret .= '</div>'; // gdlr-core-skill-circle-main-content-wrap
					}
				}
				
				$ret .= '</div>'; // gdlr-core-skill-circle-item
				
				return $ret;
			}			
			
		} // gdlr_core_pb_element_skill_circle_item
	} // class_exists	