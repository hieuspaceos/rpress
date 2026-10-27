<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('event', 'gdlr_core_pb_element_event'); 

	if( !class_exists('gdlr_core_pb_element_event') ){
		class gdlr_core_pb_element_event{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'icon_briefcase',
					'title' => esc_html__('Event (Tribe Events)', 'goodlayers-core')
				);
			}

			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(

							'category' => array(
								'title' => esc_html__('Category', 'goodlayers-core'),
								'type' => 'multi-combobox',
								'options' => gdlr_core_get_term_list('tribe_events_cat'),
								'description' => esc_html__('You can use Ctrl/Command button to select multiple items or remove the selected item. Leave this field blank to select all items in the list.', 'goodlayers-core'),
							),
							'tag' => array(
								'title' => esc_html__('Tag', 'goodlayers-core'),
								'type' => 'multi-combobox',
								'options' => gdlr_core_get_term_list('post_tag')
							),
							'num-fetch' => array(
								'title' => esc_html__('Num Fetch', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'default' => 9,
								'description' => esc_html__('The number of posts showing on the blog item', 'goodlayers-core')
							),
							'orderby' => array(
								'title' => esc_html__('Order By', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'date' => esc_html__('Event Date', 'goodlayers-core'), 
									'title' => esc_html__('Title', 'goodlayers-core'), 
									'rand' => esc_html__('Random', 'goodlayers-core'), 
									'menu_order' => esc_html__('Menu Order', 'goodlayers-core'), 
								)
							),
							'order' => array(
								'title' => esc_html__('Order', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'desc'=>esc_html__('Descending Order', 'goodlayers-core'), 
									'asc'=> esc_html__('Ascending Order', 'goodlayers-core'), 
								)
							),
							'pagination' => array(
								'title' => esc_html__('Pagination', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none'=>esc_html__('None', 'goodlayers-core'), 
									'page'=>esc_html__('Page', 'goodlayers-core'), 
								),
							),
							'pagination-style' => array(
								'title' => esc_html__('Pagination Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'default' => esc_html__('Default', 'goodlayers-core'),
									'plain' => esc_html__('Plain', 'goodlayers-core'),
									'rectangle' => esc_html__('Rectangle', 'goodlayers-core'),
									'rectangle-border' => esc_html__('Rectangle Border', 'goodlayers-core'),
									'round' => esc_html__('Round', 'goodlayers-core'),
									'round-border' => esc_html__('Round Border', 'goodlayers-core'),
									'circle' => esc_html__('Circle', 'goodlayers-core'),
									'circle-border' => esc_html__('Circle Border', 'goodlayers-core'),
								),
								'default' => 'default',
								'condition' => array( 'pagination' => 'page' )
							),
							'pagination-align' => array(
								'title' => esc_html__('Pagination Alignment', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'with-default' => true,
								'default' => 'default',
								'condition' => array( 'pagination' => 'page' )
							),
						)
					), // general

					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(	
							'event-style' => array(
								'title' => esc_html__('Event Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'widget' => esc_html__('Widget', 'goodlayers-core'),
									'widget-with-border' => esc_html__('Widget With Border', 'goodlayers-core'),
									'grid' => esc_html__('Grid', 'goodlayers-core'),
									'grid2' => esc_html__('Grid2', 'goodlayers-core'),
								),
								'default' => 'grid'
							),
							'with-frame' => array(
								'title' => esc_html__('With Frame', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable', 
								'condition' => array( 'event-style' => array('grid', 'grid2') )
							),
							'with-feature' => array(
								'title' => esc_html__('With Feature Event', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('None', 'goodlayers-core'),
									'left-feature' => esc_html__('Left Feature', 'goodlayers-core'),
									'left-feature-extend' => esc_html__('Left Feature Extend', 'goodlayers-core')
								),
								'condition' => array( 'event-style' => array('widget', 'widget-with-border') )
							),
							'column-size' => array(
								'title' => esc_html__('Column Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5 ),
								'default' => 3,
								'condition' => array('event-style' => array('grid', 'grid2'))
							),
							'thumbnail-size' => array(
								'title' => esc_html__('Thumbnail Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size',
								'condition' => array('event-style' => array('grid', 'grid2')),
							),
							'feature-thumbnail-size' => array(
								'title' => esc_html__('Thumbnail Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size',
								'condition' => array('event-style' => array('widget', 'widget-with-border'), 'with-feature' => array('left-feature', 'left-feature-extend')),
							),
						)
					),

					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'event-date-day-color' => array(
								'title' => esc_html__('Event Date Day Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'event-date-month-color' => array(
								'title' => esc_html__('Event Date Month Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'event-date-divider-color' => array(
								'title' => esc_html__('Event Date Divider Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'frame-shadow-size' => array(
								'title' => esc_html__('Shadow Size ( only for frame style )', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
							),
							'frame-shadow-color' => array(
								'title' => esc_html__('Shadow Color ( only for frame style )', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'frame-shadow-opacity' => array(
								'title' => esc_html__('Shadow Opacity ( only for frame style )', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
						)
					),

					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'frame-padding' => array(
								'title' => esc_html__('Frame Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'unlink' ),
							),
							'event-bottom-margin' => array(
								'title' => esc_html__('Event Bottom Margin ( Space Between Each Item )', 'goodlayers-core'),
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
					),

					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'title-font-size' => array(
								'title' => esc_html__('Title Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
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
								'default' => 'none'
							),
						)
					),

					'item-title' => array(
						'title' => esc_html__('Item Title', 'goodlayers-core'),
						'options' => gdlr_core_block_item_option()
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
				global $gdlr_core_item_pdb, $page_builder_post_id;
				
				// default variable
				if( empty($settings) ){
					$settings = array( 'padding-bottom' => $gdlr_core_item_pdb );
				}
				
				// generate id for sync height
				global $event_item_id;
				$event_item_id = empty($event_item_id)? 1: $event_item_id+1;
				$settings['event-item-id'] = $event_item_id;

				$settings['with-frame'] = empty($settings['with-frame'])? 'disable': $settings['with-frame'];

				// start printing item
				$ret  = '<div class="gdlr-core-event-item gdlr-core-item-pdb" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( defined('TRIBE_EVENTS_FILE') ){

					$title_settings = $settings;
					$ret .= gdlr_core_block_item_title($title_settings);

					$query = self::query_post($settings);
					$settings['paged'] = empty($query->query['paged'])? 1: $query->query['paged'];

					gdlr_core_setup_admin_postdata();
					if( $query->have_posts() ){

						$count = 0;
						$settings['event-style'] = empty($settings['event-style'])? 'grid': $settings['event-style'];
						$settings['column-size'] = empty($settings['column-size'])? 3: intval($settings['column-size']);

						$feature = false;
						$settings['with-feature'] = empty($settings['with-feature'])? '': $settings['with-feature'];
						if( in_array($settings['event-style'], array('widget', 'widget-with-border')) ){
							if( in_array($settings['with-feature'], array('left-feature', 'left-feature-extend')) ){
								$feature = 1;
							}
						}

						$ret .= '<div class="gdlr-core-event-item-holder clearfix" >';
						while($query->have_posts()){ $query->the_post(); $count++;
							$extra_class  = 'gdlr-core-item-pdlr ';

							if( in_array($settings['event-style'], array('grid', 'grid2')) ){
								$extra_class .= 'gdlr-core-style-' . $settings['event-style'] . ' ';
								$extra_class .= 'gdlr-core-column-' . (60 / $settings['column-size']) . ' ';
								$extra_class .= ($settings['with-frame'] == 'disable')? 'gdlr-core-without-frame ': 'gdlr-core-with-frame '; 
								if( $settings['column-size'] == 1 || $count % $settings['column-size'] == 1 ){
									$extra_class .= 'gdlr-core-column-first ';
								}

								$shadow_atts = array();
								if( !empty($settings['frame-shadow-size']['size']) && !empty($settings['frame-shadow-color']) && !empty($settings['frame-shadow-opacity']) ){
									$shadow_atts['background-shadow-size'] = $settings['frame-shadow-size'];
									$shadow_atts['background-shadow-color'] = $settings['frame-shadow-color'];
									$shadow_atts['background-shadow-opacity'] = $settings['frame-shadow-opacity'];
								}

								$ret .= '<div class="gdlr-core-event-item-list ' . esc_attr($extra_class) . ' clearfix" ' . gdlr_core_esc_style(array(
									'margin-bottom' => empty($settings['event-bottom-margin'])? '': $settings['event-bottom-margin']
								)) . ' >';
								$ret .= '<div class="gdlr-core-event-item-list-inner" ' . gdlr_core_esc_style($shadow_atts) . ' >';
								if( $settings['event-style'] == 'grid2' ){
									$ret .= self::get_event_grid2($settings);
								}else{
									$ret .= self::get_event_grid($settings);
								}
								$ret .= '</div>';
								$ret .= '</div>';
							}else if( in_array($settings['event-style'], array('widget', 'widget-with-border')) ){
								$extra_class .= ' gdlr-core-style-widget ';
								if( $feature == 1 ){
									$extra_class .= ' gdlr-core-event-widget-feature';
									$ret .= '<div class="gdlr-core-column-30" >';
								}
								if( $settings['event-style'] == 'widget-with-border' ){
									$extra_class .= ' gdlr-core-with-border';
								}

								$ret .= '<div class="gdlr-core-event-item-list ' . esc_attr($extra_class) . ' clearfix" ' . gdlr_core_esc_style(array(
									'margin-bottom' => empty($settings['event-bottom-margin'])? '': $settings['event-bottom-margin']
								)) . ' >';
								$ret .= self::get_event_widget($settings, ($feature == 1));
								$ret .= '</div>';

								if( $feature == 1 ){
									$ret .= '</div>';
									$ret .= '<div class="gdlr-core-column-30" >';

									$feature = 2;
								}
							}

						}
						if( $feature == 2 ){
							$ret .= '</div>'; // feature widget column
						}
						$ret .= '</div>';

						if( !empty($settings['pagination']) && $settings['pagination'] == 'page' ){
							$ret .= gdlr_core_get_pagination($query->max_num_pages, $settings, 'gdlr-core-item-pdlr');
						}
					}else{
						$ret .= '<div class="gdlr-core-course-not-found" >';
						$ret .= esc_html__('There\'re no item that match your search criteria. Please try again with different keywords.', 'goodlayers-core');
						$ret .= '</div>';
					}

					wp_reset_postdata();
					gdlr_core_reset_admin_postdata();
				}else{
					$ret .= '<div class="gdlr-core-external-plugin-message">' . esc_html__('Please activate "The Events Calendar" plugin to use this item', 'goodlayers-core') . '</div>';
				}
				
				$ret .= '</div>';


				return $ret;
			}

			static function query_post( $settings = array() ){

				if( !empty($settings['query']) ){ return $settings['query']; }

				$args = array('post_type' => 'tribe_events', 'post_status' => 'publish', 'suppress_filters' => true);
				
				// category - tag selection
				$args['tax_query'] = array('relation' => 'OR');
				
				if( !empty($settings['category']) ){
					if( !is_array($settings['category']) ){
						$settings['category'] = array_map('trim', explode(',', $settings['category']));
					}
					array_push($args['tax_query'], array('terms'=>$settings['category'], 'taxonomy'=>'tribe_events_cat', 'field'=>'slug'));
				}
				if( !empty($settings['tag']) ){
					if( !is_array($settings['tag']) ){
						$settings['tag'] = array_map('trim', explode(',', $settings['tag']));
					}
					array_push($args['tax_query'], array('terms'=>$settings['tag'], 'taxonomy'=>'post_tag', 'field'=>'slug'));
				}

				// order
				$args['order'] = empty($settings['order'])? 'desc': $settings['order'];
				$args['orderby'] = empty($settings['orderby'])? 'date': $settings['orderby'];
				if( $args['orderby'] == 'date' ){
					$args['orderby'] = 'meta_value';
					$args['meta_key'] = '_EventStartDate';
				}
				$args['meta_query'] = array(array(
					'key' => '_EventStartDate',
					'value' => date('Y-m-d') . ' 00:00:00',
					'compare' => '>=', 
					'type' => 'DATE' 
				));
				
				$args['posts_per_page'] = empty($settings['num-fetch'])? 9: $settings['num-fetch'];

				$args['paged'] = (get_query_var('paged'))? get_query_var('paged') : get_query_var('page');
				$args['paged'] = empty($args['paged'])? 1: $args['paged'];

				return new WP_Query( $args );
			}

			static function get_event_widget($settings = array(), $feature = false){
				$ret = '<div class="gdlr-core-event-widget clearfix" >';
				if( $feature ){
					$thumbnail_id = get_post_thumbnail_ID();
					if( !empty($thumbnail_id) ){
						$thumbnail_size = empty($settings['feature-thumbnail-size'])? 'full': $settings['feature-thumbnail-size'];
						$ret .= '<div class="gdlr-core-evet-widget-thumbnail gdlr-core-media-image ';
						$ret .= ($settings['with-feature'] == 'left-feature-extend')? ' gdlr-core-extend': '';
						$ret .= '" ><a href="' . esc_attr(get_permalink()) . '" >';
						$ret .= gdlr_core_get_image($thumbnail_id, $thumbnail_size);
						$ret .= '</a></div>';
						$ret .= '<div class="gdlr-core-event-widget-inner clearfix" >';
					}
				}

				$ret .= self::get_event_info(array('start-date-month'), false, $settings);
				$ret .= '<div class="gdlr-core-event-item-content-wrap" >';
				$ret .= '<h3 class="gdlr-core-event-item-title" ' . gdlr_core_esc_style(array(
					'font-size' => empty($settings['title-font-size'])? '': $settings['title-font-size'],
					'font-weight' => empty($settings['title-font-weight'])? '': $settings['title-font-weight'],
					'letter-spacing' => empty($settings['title-letter-spacing'])? '': $settings['title-letter-spacing'],
					'text-transform' => empty($settings['title-text-transform'])? '': $settings['title-text-transform'],
				)) . ' ><a href="' . get_permalink() . '" >' . get_the_title() . '</a></h3>';
				$ret .= self::get_event_info(array('time', 'location'));
				$ret .= '</div>';

				if( $feature ){
					if( !empty($thumbnail_id) ){
						$ret .= '</div>';
					}
				}
				$ret .= '</div>';

				return $ret;
			}
			static function get_event_grid($settings = array()){
				$ret = '';

				$thumbnail_id = get_post_thumbnail_id();
				if( !empty($thumbnail_id) ){
					$settings['thumbnail-size'] = empty($settings['thumbnail-size'])? 'full': $settings['thumbnail-size'];
					
					$ret .= '<div class="gdlr-core-event-item-thumbnail" ><a href="' . get_permalink() . '" >';
					$ret .= gdlr_core_get_image($thumbnail_id, $settings['thumbnail-size']);
					$ret .= '</a></div>';
				}

				if( $settings['with-frame'] == 'enable' ){
					$ret .= '<div class="gdlr-core-frame gdlr-core-skin-e-background gdlr-core-js" ' . gdlr_core_esc_style(array(
						'padding' => empty($settings['frame-padding'])? '': $settings['frame-padding']
					)) . ' data-sync-height="event-item-' . esc_attr($settings['event-item-id']) . '" >';
				}
				$ret .= self::get_event_info(array('start-date-month'), false, $settings);

				$ret .= '<div class="gdlr-core-event-item-content-wrap" >';
				$ret .= '<h3 class="gdlr-core-event-item-title" ' . gdlr_core_esc_style(array(
					'font-size' => empty($settings['title-font-size'])? '': $settings['title-font-size'],
					'font-weight' => empty($settings['title-font-weight'])? '': $settings['title-font-weight'],
					'letter-spacing' => empty($settings['title-letter-spacing'])? '': $settings['title-letter-spacing'],
					'text-transform' => empty($settings['title-text-transform'])? '': $settings['title-text-transform'],
				)) . ' ><a href="' . get_permalink() . '" >' . get_the_title() . '</a></h3>';
				
				$ret .= self::get_event_info(array('time', 'location'));

				if( $settings['with-frame'] == 'enable' ){
					$ret .= '</div>';	
				}	
				$ret .= '</div>';

				return $ret;
			}
			static function get_event_grid2($settings = array()){
				$ret = '';

				$thumbnail_id = get_post_thumbnail_id();
				if( !empty($thumbnail_id) ){
					$settings['thumbnail-size'] = empty($settings['thumbnail-size'])? 'full': $settings['thumbnail-size'];
					
					$ret .= '<div class="gdlr-core-event-item-thumbnail" ><a href="' . get_permalink() . '" >';
					$ret .= gdlr_core_get_image($thumbnail_id, $settings['thumbnail-size']);
					$ret .= '</a></div>';
				}

				if( $settings['with-frame'] == 'enable' ){
					$ret .= '<div class="gdlr-core-frame gdlr-core-skin-e-background gdlr-core-js" ' . gdlr_core_esc_style(array(
						'padding' => empty($settings['frame-padding'])? '': $settings['frame-padding']
					)) . ' data-sync-height="event-item-' . esc_attr($settings['event-item-id']) . '" >';
				}
				
				$ret .= self::get_event_info(array('start-time'), false, $settings);

				$ret .= '<div class="gdlr-core-event-item-content-wrap" >';
				$ret .= '<h3 class="gdlr-core-event-item-title" ' . gdlr_core_esc_style(array(
					'font-size' => empty($settings['title-font-size'])? '': $settings['title-font-size'],
					'font-weight' => empty($settings['title-font-weight'])? '': $settings['title-font-weight'],
					'letter-spacing' => empty($settings['title-letter-spacing'])? '': $settings['title-letter-spacing'],
					'text-transform' => empty($settings['title-text-transform'])? '': $settings['title-text-transform'],
				)) . ' ><a href="' . get_permalink() . '" >' . get_the_title() . '</a></h3>';
				
				$ret .= self::get_event_info(array('time', 'location'));

				if( $settings['with-frame'] == 'enable' ){
					$ret .= '</div>';	
				}	
				$ret .= '</div>';

				return $ret;
			}
			static function get_event_info($types = array(), $wrap = true, $settings = array() ){
				$ret = '';

				foreach( $types as $type ){
					$temp = '';
					$temp_css_atts = array();

					switch($type){
						case 'start-date-month':
							$event = get_post();
							$temp .= '<span class="gdlr-core-date" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['event-date-day-color'])? '': $settings['event-date-day-color']
							)) . ' >' . tribe_get_start_date($event, false, 'd') . '</span>'; //d
							$temp .= '<span class="gdlr-core-month" ' . gdlr_core_esc_style(array(
								'color' => empty($settings['event-date-month-color'])? '': $settings['event-date-month-color']
							)) . ' >' . tribe_get_start_date($event, false, 'M') . '</span>'; //M

							$temp_css_atts['border-color'] = empty($settings['event-date-divider-color'])? '': $settings['event-date-divider-color'];
							
							break;

						case 'start-time':
							$event = get_post();

							$temp .= '<span class="gdlr-core-tail" >' . tribe_get_start_date($event, false, 'j F Y') . '</span>';
							
							break;

						case 'time':
							$event = get_post();
							$start_date = tribe_get_start_date($event, false, 'Y m d');
							$end_date = tribe_get_start_date($event, false, 'Y m d');
							$time_format = get_option('time_format', 'g:i a');

							$temp .= '<span class="gdlr-core-head" ><i class="icon_clock_alt" ></i></span>';
								
							if( $start_date == $end_date ){
								$temp .= '<span class="gdlr-core-tail" >' . tribe_get_start_date($event, true, $time_format) . ' - ' . tribe_get_end_date($event, true, $time_format) . '</span>';
							}else{
								$temp .= '<span class="gdlr-core-tail" >' . tribe_get_start_date($event, true, $time_format) . '</span>';
							}
							break;

						case 'location':
							$location = tribe_get_venue(get_the_ID());

							if( !empty($location) ){
								$temp .= '<span class="gdlr-core-head" ><i class="icon_pin_alt" ></i></span>';
								$temp .= '<span class="gdlr-core-tail" >' . $location . '</span>';
							}
							break;

						default:
							break;
					}

					if( !empty($temp) ){
						$ret .= '<span class="gdlr-core-event-item-info gdlr-core-skin-caption gdlr-core-type-' . $type . '" ' . gdlr_core_esc_style($temp_css_atts) . ' >' . $temp . '</span>';
					}
				}
				

				if( $wrap && !empty($ret) ){
					$ret = '<div class="gdlr-core-event-item-info-wrap" >' . $ret . '</div>';
				}

				return $ret;
			}
			
		} // gdlr_core_pb_element_event
	} // class_exists	