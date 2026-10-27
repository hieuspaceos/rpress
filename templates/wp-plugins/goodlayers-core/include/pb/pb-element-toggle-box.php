<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('toggle-box', 'gdlr_core_pb_element_toggle_box'); 
	
	if( !class_exists('gdlr_core_pb_element_toggle_box') ){
		class gdlr_core_pb_element_toggle_box{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-plus-square-o',
					'title' => esc_html__('Toggle Box', 'goodlayers-core')
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
								'title' => esc_html__('Add Toggle Box Tab', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'tabs',
								'wrapper-class' => 'gdlr-core-fullsize',
								'options' => array(
									'head-text' => array(
										'title' => esc_html__('Head Text', 'goodlayers-core'),
										'type' => 'text'
									),
									'title' => array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content' => array(
										'title' => esc_html__('Content', 'goodlayers-core'),
										'type' => 'tmce'
									),
									'active' => array(
										'title' => esc_html__('Open By Default', 'goodlayers-core'),
										'type' => 'checkbox'
									),
								),
								'default' => array(
									array(
										'title' => esc_html__('A Toggle Box Title', 'goodlayers-core'),
										'content' => esc_html__('A toggle box content area', 'goodlayers-core'),
										'active' => 'enable',
									),
									array(
										'title' => esc_html__('A Toggle Box Title', 'goodlayers-core'),
										'content' => esc_html__('A toggle box content area', 'goodlayers-core'),
										'active' => 'disable',
									),
								)
							),
						),
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'style' => array(
								'title' => esc_html__('Toggle Box Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'box-icon' => GDLR_CORE_URL . '/include/images/accordion-toggle/box-icon.png',
									'icon' => GDLR_CORE_URL . '/include/images/accordion-toggle/icon.png',
									'background-title' => GDLR_CORE_URL . '/include/images/accordion-toggle/background-title.png',
									'background-title-icon' => GDLR_CORE_URL . '/include/images/accordion-toggle/background-title-icon.png',	
									'box-background' => GDLR_CORE_URL . '/include/images/accordion-toggle/box-background.jpg',
									'shadow-background' =>  GDLR_CORE_URL . '/include/images/accordion-toggle/shadow-background.jpg',
								),
								'default' => 'box-icon',
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'with-divider' => array(
								'title' => esc_html__('With Divider', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'style' => 'icon' )
							),
							'align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'condition' => array( 'style' => array('background-title', 'background-title-icon', 'box-background') ),
								'default' => 'left'
							),
							'icon-position' => array(
								'title' => esc_html__('Icon Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'left' => esc_html__('Left', 'goodlayers-core'),
									'right' => esc_html__('Right', 'goodlayers-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
								),
								'condition' => array( 'style' => array('icon', 'background-title-icon', 'box-background') )
							),
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'title-font-size' => array(
								'title' => esc_html__('Title Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
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
								'data-input-type' => 'pixel'
							),
							'title-text-transform' => array(
								'title' => esc_html__('Title Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'default' => esc_html__('Default', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
								),
							),
							'content-font-size' => array(
								'title' => esc_html__('Content Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'title-padding' => array(
								'title' => esc_html__('Title Padding (Background Style)', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'unlink' ),
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
?><script id="gdlr-core-preview-toggle-box-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-toggle-box-<?php echo esc_attr($id); ?>').parent().gdlr_core_toggle_box();
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
						'tabs' => array(
							array(
								'title' => esc_html__('A Toggle Box Title', 'goodlayers-core'),
								'content' => esc_html__('A toggle box content area', 'goodlayers-core'),
								'active' => 'enable',
							),
							array(
								'title' => esc_html__('A Toggle Box Title', 'goodlayers-core'),
								'content' => esc_html__('A toggle box content area', 'goodlayers-core'),
								'active' => 'enable',
							),
						),
						'style' => 'box-icon',
						'align' => 'left',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				$settings['style'] = empty($settings['style'])? 'box-icon': $settings['style'];
				
				// start printing item
				$icon_class = ($settings['style'] == 'box-icon')? ' gdlr-core-skin-e-background gdlr-core-skin-border': '';
				$title_class = in_array($settings['style'], array('background-title', 'background-title-icon', 'box-background'))? ' gdlr-core-skin-e-background gdlr-core-skin-e-content': '';
				$toggle_item_class  = ' gdlr-core-toggle-box-style-' . esc_attr($settings['style']);
				$toggle_item_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				if( in_array($settings['style'], array('background-title', 'background-title-icon', 'box-background')) && !empty($settings['align']) ){
					$toggle_item_class .= ' gdlr-core-' . esc_attr($settings['align']) . '-align';
				}
				if( in_array($settings['style'], array('icon', 'background-title-icon', 'box-background')) ){
					$toggle_item_class .= ' gdlr-core-icon-pos-' . (empty($settings['icon-position'])? 'left': $settings['icon-position']);
				}
				if( $settings['style'] == 'icon' && !empty($settings['with-divider']) && $settings['with-divider'] == 'enable' ){
					$toggle_item_class .= ' gdlr-core-with-divider';
				}
				
				$ret  = '<div class="gdlr-core-toggle-box-item gdlr-core-item-mglr gdlr-core-item-mgb ' . esc_attr($toggle_item_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('marign-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				if( !empty($settings['tabs']) ){
					foreach( $settings['tabs'] as $tab ){
						$title_atts = array(
							'font-size' => empty($settings['title-font-size'])? '': $settings['title-font-size'],
							'font-style' => empty($settings['title-font-style'])? '': $settings['title-font-style'],
							'font-weight' => empty($settings['title-font-weight'])? '': $settings['title-font-weight'],
							'letter-spacing' => empty($settings['title-letter-spacing'])? '': $settings['title-letter-spacing'],
							'text-transform' => (empty($settings['title-text-transform']) || $settings['title-text-transform'] == 'default')? '': $settings['title-text-transform']
						);
						if( in_array($settings['style'], array('background', 'background-title-icon', 'box-background')) ){
							if( !empty($settings['title-padding']) && $settings['title-padding'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'unlink' ) ){
								$title_atts['padding'] = $settings['title-padding'];
							}
						}

						$ret .= '<div class="gdlr-core-toggle-box-item-tab clearfix ' . ($tab['active'] == 'enable'? ' gdlr-core-active': '') . '" >';
						$ret .= '<div class="gdlr-core-toggle-box-item-icon gdlr-core-js gdlr-core-skin-icon ' . esc_attr($icon_class) . '"></div>';
						$ret .= '<div class="gdlr-core-toggle-box-item-content-wrapper">';
						$ret .= '<h4 class="gdlr-core-toggle-box-item-title gdlr-core-js ' . esc_attr($title_class) . '" ' . gdlr_core_esc_style($title_atts) . ' >';
						if( !empty($tab['head-text']) ){
							$ret .= '<span class="gdlr-core-head" >' . $tab['head-text'] . '</span>';
						}
						$ret .= gdlr_core_text_filter($tab['title']);
						$ret .= '</h4>';
						$ret .= '<div class="gdlr-core-toggle-box-item-content" ' . gdlr_core_esc_style(array(
							'font-size' => empty($settings['content-font-size'])? '': $settings['content-font-size']
						)) . ' >' . gdlr_core_content_filter($tab['content']) . '</div>';
						$ret .= '</div>'; // gdlr-core-toggle-box-item-content-wrapper
						$ret .= '</div>'; // gdlr-core-toggle-box-item-tab
					}
				}
				$ret .= '</div>'; // gdlr-core-toggle-box-item
				
				return $ret;
			}			
			
		} // gdlr_core_pb_element_column_service
	} // class_exists	