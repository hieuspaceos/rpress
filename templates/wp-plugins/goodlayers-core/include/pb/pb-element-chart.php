<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('chart', 'gdlr_core_pb_element_chart'); 
	
	if( !class_exists('gdlr_core_pb_element_chart') ){
		class gdlr_core_pb_element_chart{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-signal',
					'title' => esc_html__('Chart', 'goodlayers-core')
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
									'data' => array(
										'title' => esc_html__('Data ( Fill only number )', 'goodlayers-core'),
										'type' => 'text'
									),
									'color' => array(
										'title' => esc_html__('Color ( Omitted for Line Style )', 'goodlayers-core'),
										'type' => 'colorpicker'
									),
								),
								'default' => array(
									array(
										'title' => esc_html__('Tab Title', 'goodlayers-core'),
										'data' => 25,
										'color' => '#ff6384',
									),
									array(
										'title' => esc_html__('Tab Title', 'goodlayers-core'),
										'data' => 25,
										'color' => '#36a2eb',
									),
									array(
										'title' => esc_html__('Tab Title', 'goodlayers-core'),
										'data' => 25,
										'color' => '#cc65fe',
									),
									array(
										'title' => esc_html__('Tab Title', 'goodlayers-core'),
										'data' => 25,
										'color' => '#ffce56',
									),
								)
							),
						),
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'type' => array(
								'title' => esc_html__('Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'pie' => esc_html__('Pie', 'goodlayers-core'),
									'doughnut' => esc_html__('Doughnut', 'goodlayers-core'),
									'line' => esc_html__('Line', 'goodlayers-core'),
									'bar' => esc_html__('Bar Vertical', 'goodlayers-core'),
									'horizontalBar' => esc_html__('Bar Horizontal', 'goodlayers-core'),
								)
							),
							'legend-position' => array(
								'title' => esc_html__('Legend Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'top' => esc_html__('Top', 'goodlayers-core'),
									'right' => esc_html__('Right', 'goodlayers-core'),
									'bottom' => esc_html__('Bottom', 'goodlayers-core'),
									'left' => esc_html__('Left', 'goodlayers-core'),
								),
								'condition' => array('type' => array('pie', 'doughnut'))
							),
							'legend-text-color' => array(
								'title' => esc_html__('Legend Text Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array('type' => array('pie', 'doughnut'))
							),
							'border-color' => array(
								'title' => esc_html__('Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array('type' => array('pie', 'doughnut', 'line'))
							),
							'hover-border-color' => array(
								'title' => esc_html__('Hover Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array('type' => array('pie', 'doughnut'))
							),
							'grid-lines' => array(
								'title' => esc_html__('Grid Lines Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array('type' => array('line', 'bar', 'horizontalBar'))
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
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-chart-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-chart-<?php echo esc_attr($id); ?>').parent().gdlr_core_chart_js();
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
								'title' => esc_html__('Tab Title', 'goodlayers-core'),
								'data' => 25,
								'color' => '#ff6384',
							),
							array(
								'title' => esc_html__('Tab Title', 'goodlayers-core'),
								'data' => 25,
								'color' => '#36a2eb',
							),
							array(
								'title' => esc_html__('Tab Title', 'goodlayers-core'),
								'data' => 25,
								'color' => '#cc65fe',
							),
							array(
								'title' => esc_html__('Tab Title', 'goodlayers-core'),
								'data' => 25,
								'color' => '#ffce56',
							),
						),
						'type' => 'pie',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				$settings['type'] = empty($settings['type'])? 'pie': $settings['type'];

				// start printing item
				$ret  = '<div class="gdlr-core-chart-item gdlr-core-item-pdlr gdlr-core-item-pdb" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				// obtain data from tabs
				if( !empty($settings['tabs']) ){
					$options = array();

					$data = array();
					$dataset = array();
					$labels = array();
					$backgrounds = array();
					foreach( $settings['tabs'] as $tab ){
						$dataset[] = empty($tab['data'])? '': $tab['data'];
						$labels[] = empty($tab['title'])? '': $tab['title'];
						$backgrounds[] = empty($tab['color'])? '': $tab['color'];
					}

					// pie/doughnut
					if( in_array($settings['type'], array('pie', 'doughnut')) ){
						$borderColors = array();
						$hoverBorderColors = array();
						for( $i = 0; $i < sizeof($settings['tabs']); $i++ ){
							if( !empty($settings['border-color']) ){
								$borderColors[] = $settings['border-color'];
							}
							if( !empty($settings['hover-border-color']) ){
								$hoverBorderColors[] = $settings['hover-border-color'];
							}
						}
						$data = array(
							'datasets' => array(array(
								'data' => $dataset,
								'backgroundColor' => $backgrounds,
								'borderColor' => $borderColors,
								'hoverBorderColor' => $hoverBorderColors
							)),
							'labels' => $labels
						);
						$options = array(
							'plugins' => array(
								'legend' => array(
									'position' => empty($settings['legend-position'])? 'top': $settings['legend-position'],
								),
							),
						);

						if( !empty($settings['legend-text-color']) ){
							$options['plugins']['legend']['labels']['color'] = $settings['legend-text-color'];
						}
						if( in_array($settings['legend-position'], array('left', 'right')) ){
							$options['legend']['labels']['padding'] = 18;
						}
						
					// line style		
					}else if( $settings['type'] == 'line' ){
						$data = array(
							'datasets' => array(array(
								'data' => $dataset,
								'fill' => false,
								'lineTension' => 0.1,
								'borderColor' => empty($settings['border-color'])? '': $settings['border-color']
							)),
							'labels' => $labels,
						);
						$options = array(
							'legend' => array(
								'display' => false
							),
							'scales' => array(
								'yAxes' => array(array(
									'ticks' => array(
										'beginAtZero' => true
									)
								)),
							)
						);

					// bar style
					}else if( in_array($settings['type'], array('bar', 'horizontalBar')) ){

						$borders = $backgrounds;
						$backgrounds = array();
						foreach($borders as $border){
							$backgrounds[] = 'rgba(' . gdlr_core_format_datatype($border, 'rgba') . ', 0.4)';
						}
						$data = array(
							'datasets' => array(array(
								'data' => $dataset,
								'backgroundColor' => $backgrounds,
								'borderColor' => $borders,
								'borderWidth' => 1,
							)),
							'labels' => $labels
						);
						
						$axes = ($settings['type'] == 'bar')? 'yAxes': 'xAxes';
						$options = array(
							'legend' => array(
								'display' => false
							),
							'scales' => array(
								$axes => array(array(
									'ticks' => array(
										'beginAtZero' => true
									)
								))
							)
						);
					}
					
					// set gridlines color
					if( in_array($settings['type'], array('line', 'bar', 'horizontalBar')) && !empty($settings['grid-lines']) ){
						$options['scales']['xAxes'][0]['gridLines']['color'] = $settings['grid-lines'];
						$options['scales']['xAxes'][0]['ticks']['fontColor'] = $settings['grid-lines'];
						$options['scales']['yAxes'][0]['gridLines']['color'] = $settings['grid-lines'];
						$options['scales']['yAxes'][0]['ticks']['fontColor'] = $settings['grid-lines'];
					}


					$ret .= '<canvas class="gdlr-core-chart-js gdlr-core-js" ';
					$ret .= 'data-type="' . esc_attr($settings['type']) . '" ';
					$ret .= 'data-content="' . esc_attr(json_encode($data)) . '" ';
					$ret .= 'data-options="' . esc_attr(json_encode($options)) . '" ></canvas>';
				}else{
					$ret .= esc_html__('Please edit an item to add new dataset', 'goodlayers-core');
				}

				$ret .= '</div>'; // gdlr-core-chart-item
				
				return $ret;
			}			
			
		} // gdlr_core_pb_element_chart
	} // class_exists	