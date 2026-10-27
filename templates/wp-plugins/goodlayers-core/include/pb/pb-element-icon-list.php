<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('icon-list', 'gdlr_core_pb_element_icon_list_item'); 
	
	if( !class_exists('gdlr_core_pb_element_icon_list_item') ){
		class gdlr_core_pb_element_icon_list_item{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-list-ul',
					'title' => esc_html__('Icon List', 'goodlayers-core')
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
								'title' => esc_html__('Icon List', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'tabs',
								'wrapper-class' => 'gdlr-core-fullsize',
								'options' => array(
									'image' => array(
										'title' => esc_html__('Image', 'goodlayers-core'),
										'type' => 'upload'
									),
									'icon' => array(
										'title' => esc_html__('Icon', 'goodlayers-core'),
										'type' => 'text',
										'data-input-type' => 'icon'
									),
									'icon-hover' => array(
										'title' => esc_html__('Icon Hover', 'goodlayers-core'),
										'type' => 'text',
										'data-input-type' => 'icon'
									), 
									'icon-size' => array(
										'title' => esc_html__('Icon Size', 'goodlayers-core'),
										'type' => 'text',
										'data-input-type' => 'pixel'
									),
									'icon-top-margin' => array(
										'title' => esc_html__('Icon Top Margin', 'goodlayers-core'),
										'type' => 'text',
										'data-input-type' => 'pixel'
									),
									'title' => array(
										'title' => esc_html__('Content', 'goodlayers-core'),
										'type' => 'text',
									),  
									'caption' => array(
										'title' => esc_html__('Caption', 'goodlayers-core'),
										'type' => 'text',
									),  
									'link-url' => array(
										'title' => esc_html__('Link Url', 'goodlayers-core'),
										'type' => 'text',
									),   
									'link-target' => array(
										'title' => esc_html__('Link Target', 'goodlayers-core'),
										'type' => 'combobox',
										'options' => array(
											'_self' => esc_html__('Current Screen', 'goodlayers-core'),
											'_blank' => esc_html__('New Window', 'goodlayers-core'),
										)
									), 
								),
								'default' => array(
									array(
										'icon' => 'fa fa-gear',
										'icon-hover' => 'fa fa-flag',
										'title' => esc_html__('Default icon list text', 'goodlayers-core'),
									),
									array(
										'icon' => 'fa fa-gear',
										'icon-hover' => 'fa fa-flag',
										'title' => esc_html__('Default icon list text', 'goodlayers-core'),
									),
									array(
										'icon' => 'fa fa-gear',
										'icon-hover' => 'fa fa-flag',
										'title' => esc_html__('Default icon list text', 'goodlayers-core'),
									),
								)
							),
							'columns' => array(
								'title' => esc_html__('Columns', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'60' => esc_html__('1', 'goodlayers-core'),
									'30' => esc_html__('2', 'goodlayers-core'),
									'20' => esc_html__('3', 'goodlayers-core'),
									'15' => esc_html__('4', 'goodlayers-core'),
									'12' => esc_html__('5', 'goodlayers-core'),
									'10' => esc_html__('6', 'goodlayers-core'),
									'flow' => esc_html__('Auto Flow', 'goodlayers-core')
								),
								'default' => 60 
							),
						),
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'style' => array(
								'title' => esc_html__('Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'style-1' => esc_html__('Style 1', 'goodlayers-core'),
									'style-2' => esc_html__('Style 2', 'goodlayers-core'),
									'style-3' => esc_html__('Style 3', 'goodlayers-core')
								)
							),
							'text-align' => array(
								'title' => esc_html__('Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'condition' => array( 'style' => 'style-3' )
							),
							'align' => array(
								'title' => esc_html__('Align', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'left' => esc_html__('Left', 'goodlayers-core'),
									'right' => esc_html__('Right', 'goodlayers-core'),
								),
								'condition' => array( 'style' => array('style-1', 'style-2') )
							),
							'enable-divider' => array(
								'title' => esc_html__('Enable Divider', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'icon-background' => array(
								'title' => esc_html__('Icon Background', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'round' => esc_html__('Round', 'goodlayers-core'),
									'circle' => esc_html__('Circle', 'goodlayers-core'),
								)
							),
							'icon-position' => array(
								'title' => esc_html__('Icon Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'left' => esc_html__('Left ( Right For Left Align )', 'goodlayers-core'),
									'right' => esc_html__('Right ( Left For Right Align )', 'goodlayers-core'),
								),
							),
							'icon-color' => array(
								'title' => esc_html__('Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'icon-background-color' => array(
								'title' => esc_html__('Icon Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'icon-background' => array('round','circle') )
							),
							'content-color' => array(
								'title' => esc_html__('Content Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'caption-color' => array(
								'title' => esc_html__('Caption Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'border-color' => array(
								'title' => esc_html__('Divider Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'enable-divider' => 'enable' )
							),
						),
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'icon-size' => array(
								'title' => esc_html__('Icon Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '14px'
							),
							'content-size' => array(
								'title' => esc_html__('Content Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '14px'
							),
							'content-font-weight' => array(
								'title' => esc_html__('Content Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'default' => '',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'content-text-transform' => array(
								'title' => esc_html__('Content Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'' => esc_html__('Default', 'goodlayers-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'default'
							),
							'content-letter-spacing' => array(
								'title' => esc_html__('Content Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'caption-size' => array(
								'title' => esc_html__('Caption Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '14px'
							),
							'caption-font-weight' => array(
								'title' => esc_html__('Caption Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'default' => '',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'caption-text-transform' => array(
								'title' => esc_html__('Caption Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'' => esc_html__('Default', 'goodlayers-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'default'
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
							'image-max-width' => array(
								'title' => esc_html__('Image Max Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => ''
							), 
							'icon-top-margin' => array(
								'title' => esc_html__('Icon/Image Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '',
								'description' => esc_html__('Default value is 3px', 'goodlayers-core')
							), 
							'icon-right-margin' => array(
								'title' => esc_html__('Icon/Image Right Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => ''
							), 
							'list-bottom-margin' => array(
								'title' => esc_html__('List Margin ( Between Items )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '10px'
							), 
							'item-max-width' => array(
								'title' => esc_html__('Item Max Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
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
?><script id="gdlr-core-preview-skill-bar-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-skill-bar-<?php echo esc_attr($id); ?>').parent().gdlr_core_skill_bar();
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
								'icon' => 'fa fa-gear',
								'icon-hover' => 'fa fa-flag',
								'title' => esc_html__('Default icon list text', 'goodlayers-core'),
							),
							array(
								'icon' => 'fa fa-gear',
								'icon-hover' => 'fa fa-flag',
								'title' => esc_html__('Default icon list text', 'goodlayers-core'),
							),
							array(
								'icon' => 'fa fa-gear',
								'icon-hover' => 'fa fa-flag',
								'title' => esc_html__('Default icon list text', 'goodlayers-core'),
							),
						),

						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				// default size
				$settings['style'] = empty($settings['style'])? 'style-1': $settings['style'];
				$settings['icon-size'] = (empty($settings['icon-size']) || $settings['icon-size'] == '14px')? '': $settings['icon-size'];
				$settings['content-size'] = (empty($settings['content-size']) || $settings['content-size'] == '14px')? '': $settings['content-size'];
				$settings['caption-size'] = (empty($settings['caption-size']) || $settings['caption-size'] == '14px')? '': $settings['caption-size'];
				$settings['icon-background'] = empty($settings['icon-background'])? 'none': $settings['icon-background'];
				$settings['align'] = empty($settings['align'])? 'left': $settings['align'];
				if( $settings['style'] == 'style-3' ){
					$settings['align'] = empty($settings['text-align'])? 'left': $settings['text-align'];
				}
				
				// start printing item
				$extra_class  = (empty($settings['enable-divider']) || $settings['enable-divider'] == 'disable')? '': 'gdlr-core-with-divider';
				$extra_class .= ($settings['icon-background'] != 'none')? ' gdlr-core-icon-list-with-background-' . $settings['icon-background']: '';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$extra_class .= ' gdlr-core-' . $settings['align'] . '-align';
				$extra_class .= ' gdlr-core-' . $settings['style']; 
				$ret  = '<div class="gdlr-core-icon-list-item gdlr-core-item-pdlr gdlr-core-item-pdb clearfix ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				
				$count = 0;

				$ul_class = '';
				$ul_atts = array( 'max-width' => empty($settings['item-max-width'])? '': $settings['item-max-width'] );
				$li_atts = array( 'border-color' => empty($settings['border-color'])? '': $settings['border-color'] );

				if( !empty($settings['columns']) && $settings['columns'] == 'flow' ){
					$ul_class = 'gdlr-core-icon-list-flow';
					$ul_atts['gap'] = (empty($settings['list-bottom-margin']) || $settings['list-bottom-margin'] == '10px')? '': $settings['list-bottom-margin'];
				}else if( empty($settings['enable-divider']) || $settings['enable-divider'] == 'disable' ){
					$li_atts['margin-bottom'] = (empty($settings['list-bottom-margin']) || $settings['list-bottom-margin'] == '10px')? '': $settings['list-bottom-margin'];
				}else{
					$li_atts['padding-bottom'] = (empty($settings['list-bottom-margin']) || $settings['list-bottom-margin'] == '10px')? '': $settings['list-bottom-margin'];
					$li_atts['padding-top'] = $li_atts['padding-bottom'];
				}

				$ret .= '<ul class="' . esc_attr($ul_class) . '" ' . gdlr_core_esc_style($ul_atts) . ' >';
				if( !empty($settings['tabs']) ){
					foreach( $settings['tabs'] as $tab ){
						$li_class  = ' gdlr-core-skin-divider';
						$li_class .= empty($tab['icon-hover'])? '': ' gdlr-core-with-hover';

						if( !empty($settings['columns']) && !($settings['columns'] == 60 || $settings['columns'] == 'flow') ){
							$li_class .= ' gdlr-core-column-' . $settings['columns'];

							if( $count == 0 || $count == 60 ){
								$count = 0;
								$li_class .= ' gdlr-core-column-first';
							}
							$count += intval($settings['columns']);
						}

						$ret .= '<li class="' . esc_attr($li_class) . ' clearfix" ' . gdlr_core_esc_style($li_atts) . ' >';
						if( !empty($tab['link-url']) ){
							$ret .= '<a href="' . esc_url($tab['link-url']) . '" ';
							$ret .= empty($tab['link-target'])? '': 'target="' . esc_attr($tab['link-target']) . '" ';
							$ret .= '>';
						}
						if( !empty($tab['icon']) ){
							$icon_atts = array(
								'color' => empty($settings['icon-color'])? '': $settings['icon-color'],
								'font-size' => empty($tab['icon-size'])? $settings['icon-size']: $tab['icon-size'],
								'margin-top' => empty($tab['icon-top-margin'])? '': $tab['icon-top-margin'],
							);

							$icon_skin_class = '';
							$icon_margin = ($settings['align'] == 'left')? 'margin-right': 'margin-left';
							if( $settings['icon-background'] != 'none' ){
								$icon_skin_class = ' gdlr-core-skin-e-content';

								$ret .= '<span class="gdlr-core-icon-list-icon-wrap gdlr-core-skin-e-background gdlr-core-' . (empty($settings['icon-position'])? 'left': $settings['icon-position']) . '" ' . gdlr_core_esc_style(array(
									'background-color' => empty($settings['icon-background-color'])? '': $settings['icon-background-color'],
									'margin-top' => empty($settings['icon-top-margin'])? '': $settings['icon-top-margin'],
									$icon_margin => empty($settings['icon-right-margin'])? '': $settings['icon-right-margin'],
								)) . '>';
							}else{
								$ret .= '<span class="gdlr-core-icon-list-icon-wrap gdlr-core-' . (empty($settings['icon-position'])? 'left': $settings['icon-position']) . '" ' . gdlr_core_esc_style(array(
									'margin-top' => empty($settings['icon-top-margin'])? '': $settings['icon-top-margin'],
									$icon_margin => empty($settings['icon-right-margin'])? '': $settings['icon-right-margin'],
								)) . ' >'; 
							}

							if( !empty($tab['icon-hover']) ){
								$ret .= '<i class="gdlr-core-icon-list-icon-hover ' . esc_attr($tab['icon-hover']) . esc_attr($icon_skin_class) . '" ' . gdlr_core_esc_style($icon_atts) . ' ></i>';
							}
							$icon_atts['width'] = empty($tab['icon-size'])? $settings['icon-size']: $tab['icon-size'];
							$ret .= '<i class="gdlr-core-icon-list-icon ' . esc_attr($tab['icon']) . esc_attr($icon_skin_class) . '" ' . gdlr_core_esc_style($icon_atts) . '></i>';
							$ret .= '</span>';
						}
						if( !empty($tab['image']) ){
							$ret .= '<span class="gdlr-core-icon-list-image gdlr-core-' . (empty($settings['icon-position'])? 'left': $settings['icon-position']) . '" ' . gdlr_core_esc_style(array(
								'max-width' => empty($settings['image-max-width'])? '': $settings['image-max-width'],
								'margin-top' => empty($settings['icon-top-margin'])? '': $settings['icon-top-margin'],
								'margin-right' => empty($settings['icon-right-margin'])? '': $settings['icon-right-margin'],
							)) . ' >' . gdlr_core_get_image($tab['image']) . '</span>';
						}
						$ret .= '<div class="gdlr-core-icon-list-content-wrap" >';
						if( !empty($tab['title']) ){
							$ret .= '<span class="gdlr-core-icon-list-content" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['content-color'])? '': $settings['content-color'],
								'font-size' => $settings['content-size'],
								'font-weight' => empty($settings['content-font-weight'])? '': $settings['content-font-weight'],
								'text-transform' => empty($settings['content-text-transform'])? '': $settings['content-text-transform'],
								'letter-spacing' => empty($settings['content-letter-spacing'])? '': $settings['content-letter-spacing'],
							)) . ' >' . gdlr_core_text_filter($tab['title']) . '</span>';
						}
						if( !empty($tab['caption']) ){
							$ret .= '<span class="gdlr-core-icon-list-caption" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['caption-color'])? '': $settings['caption-color'],
								'font-size' => $settings['caption-size'],
								'font-weight' => empty($settings['caption-font-weight'])? '': $settings['caption-font-weight'],
								'text-transform' => empty($settings['caption-text-transform'])? '': $settings['caption-text-transform'],
								'letter-spacing' => empty($settings['caption-letter-spacing'])? '': $settings['caption-letter-spacing'],
							)) . ' >' . gdlr_core_text_filter($tab['caption']) . '</span>';
						}
						$ret .= '</div>';
						if( !empty($tab['link-url']) ){
							$ret .= '</a>';
						}
						$ret .= '</li>';
					}
				}
				$ret .= '</ul>';
				
				$ret .= '</div>'; // gdlr-core-icon-list-item
				
				return $ret;
			}			
			
		} // gdlr_core_pb_element_icon_list_item
	} // class_exists	