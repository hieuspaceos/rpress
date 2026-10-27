<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('opening-hours', 'gdlr_core_pb_element_opening_hours'); 
	
	if( !class_exists('gdlr_core_pb_element_opening_hours') ){
		class gdlr_core_pb_element_opening_hours{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'icon_clock',
					'title' => esc_html__('Opening Hours', 'goodlayers-core')
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
										'title' => esc_html__('Day', 'goodlayers-core'),
										'type' => 'text'
									),
									'time' => array(
										'title' => esc_html__('Time', 'goodlayers-core'),
										'type' => 'text'
									),
									'open' => array(
										'title' => esc_html__('Open', 'goodlayers-core'),
										'type' => 'checkbox'
									),
								),
								'default' => array(
									array(
										'title' => esc_html__('Monday', 'goodlayers-core'),
										'time' => '09:00 - 18:00',
										'open' => 'enable'
									),
									array(
										'title' => esc_html__('Tuesday', 'goodlayers-core'),
										'time' => '09:00 - 18:00',
										'open' => 'enable'
									),
									array(
										'title' => esc_html__('Wednesday', 'goodlayers-core'),
										'time' => '09:00 - 18:00',
										'open' => 'disable'
									),
									array(
										'title' => esc_html__('Thursday', 'goodlayers-core'),
										'time' => '09:00 - 18:00',
										'open' => 'enable'
									),
									array(
										'title' => esc_html__('Friday', 'goodlayers-core'),
										'time' => '09:00 - 18:00',
										'open' => 'enable'
									),
									array(
										'title' => esc_html__('Saturday', 'goodlayers-core'),
										'time' => '09:00 - 18:00',
										'open' => 'enable'
									),
									array(
										'title' => esc_html__('Sunday', 'goodlayers-core'),
										'time' => '09:00 - 18:00',
										'open' => 'disable'
									),
								)
							),

						)
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(	
							'divider-style' => array(
								'title' => esc_html__('Divider Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => array(
									'solid' => GDLR_CORE_URL . '/include/images/divider/solid.png',
									'dotted' => GDLR_CORE_URL . '/include/images/divider/dotted.png',
									'dashed' => GDLR_CORE_URL . '/include/images/divider/dashed.png'
								),
								'default' => 'solid',
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'icon-position' => array(
								'title' => esc_html__('Icon Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'left' => esc_html__('Left', 'goodlayers-core'),
									'right' => esc_html__('Right', 'goodlayers-core')
								),
								'default' => 'right'
							),
							'icon-style' => array(
								'title' => esc_html__('Icon Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Default', 'goodlayers-core'),
									'none' => esc_html__('None', 'goodlayers-core'),
									'custom' => esc_html__('Custom', 'goodlayers-core')
								)
							),
							'icon' => array(
								'title' => esc_html__('Icons', 'goodlayers-core'),
								'type' => 'icons',
								'wrapper-class' => 'gdlr-core-fullsize',
								'condition' => array( 'icon-style' => 'custom' )
							),
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'day-text-color' => array(
								'title' => esc_html__('Day Text Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'time-color' => array(
								'title' => esc_html__('Time Text Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'close-color' => array(
								'title' => esc_html__('Closed Text Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'icon-color' => array(
								'title' => esc_html__('Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'divider-color' => array(
								'title' => esc_html__('Divider Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							)
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
								'title' => esc_html__('Monday', 'goodlayers-core'),
								'time' => '09:00 - 18:00',
								'open' => 'enable'
							),
							array(
								'title' => esc_html__('Tuesday', 'goodlayers-core'),
								'time' => '09:00 - 18:00',
								'open' => 'enable'
							),
							array(
								'title' => esc_html__('Wednesday', 'goodlayers-core'),
								'time' => '09:00 - 18:00',
								'open' => 'disable'
							),
							array(
								'title' => esc_html__('Thursday', 'goodlayers-core'),
								'time' => '09:00 - 18:00',
								'open' => 'enable'
							),
							array(
								'title' => esc_html__('Friday', 'goodlayers-core'),
								'time' => '09:00 - 18:00',
								'open' => 'enable'
							),
							array(
								'title' => esc_html__('Saturday', 'goodlayers-core'),
								'time' => '09:00 - 18:00',
								'open' => 'enable'
							),
							array(
								'title' => esc_html__('Sunday', 'goodlayers-core'),
								'time' => '09:00 - 18:00',
								'open' => 'disable'
							),
						),
						'text-align' => 'left',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				$settings['divider-style'] = empty($settings['divider-style'])? 'dashed': $settings['divider-style'];
				$settings['day-text-color'] = empty($settings['day-text-color'])? '': $settings['day-text-color'];
				$settings['time-color'] = empty($settings['time-color'])? '': $settings['time-color'];
				$settings['close-color'] = empty($settings['close-color'])? '': $settings['close-color'];
				$settings['icon-color'] = empty($settings['icon-color'])? '': $settings['icon-color'];
				$settings['divider-color'] = empty($settings['divider-color'])? '': $settings['divider-color'];

				// start printing item
				$extra_class  = ' gdlr-core-divider-style-' . $settings['divider-style'];
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-opening-hour-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				$ret .= '<div class="gdlr-core-opening-hour-list" >';
				if( !empty($settings['tabs']) ){
					foreach( $settings['tabs'] as $tab ){
						$icon = '';
						if( !empty($settings['icon-style']) && $settings['icon-style'] == 'custom' && !empty($settings['icon']) ){
							$icon = '<i class="' . esc_attr($settings['icon']) . '" ' . gdlr_core_esc_style(array('color' => $settings['icon-color'])) . ' ></i>';
						}else if( empty($settings['icon-style']) || $settings['icon-style'] != 'none' ){
							if( !empty($tab['open']) && $tab['open'] == 'enable' ){
								$icon = '<i class="icon_clock_alt" ' . gdlr_core_esc_style(array('color' => $settings['icon-color'])) . ' ></i>';
							}else{
								$icon = '<i class="icon_close_alt2" ' . gdlr_core_esc_style(array('color' => $settings['icon-color'])) . ' ></i>';
							}
						}

						$ret .= '<div class="gdlr-core-opening-hour-list-item" ' . gdlr_core_esc_style(array('border-color' => $settings['divider-color'])) . ' >';
						$ret .= '<span class="gdlr-core-opening-hour-day" ' . gdlr_core_esc_style(array('color' => $settings['day-text-color'])) . ' >';
						if( !empty($icon) && !empty($settings['icon-position']) && $settings['icon-position'] == 'left' ){
							$ret .= $icon;
						}
						$ret .= gdlr_core_escape_content($tab['title']);
						$ret .= '</span>';

						if( !empty($tab['open']) && $tab['open'] == 'enable' ){
							$ret .= '<span class="gdlr-core-opening-hour-time gdlr-core-opening-hour-open" ' . gdlr_core_esc_style(array('color' => $settings['time-color'])) . ' >';
							$ret .= $tab['time'];
							if( !empty($icon) && (empty($settings['icon-position']) || $settings['icon-position'] == 'right') ){
								$ret .= $icon;
							}
							$ret .= '</span>';
						}else{
							$ret .= '<span class="gdlr-core-opening-hour-time gdlr-core-opening-hour-close" ' . gdlr_core_esc_style(array('color' => $settings['close-color'])) . ' >';
							$ret .= esc_html__('Closed', 'goodlayers-core');
							if( !empty($icon) && (empty($settings['icon-position']) || $settings['icon-position'] == 'right') ){
								$ret .= $icon;
							}
							$ret .= '</span>';
						}
						$ret .= '</div>';
					}
				}
				$ret .= '</div>'; // gdlr-core-opening-hour-list

				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_opening_hours
	} // class_exists	