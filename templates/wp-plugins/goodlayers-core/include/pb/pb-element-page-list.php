<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('page-list', 'gdlr_core_pb_element_page_list'); 
	
	if( !class_exists('gdlr_core_pb_element_page_list') ){
		class gdlr_core_pb_element_page_list{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-newspaper-o',
					'title' => esc_html__('Page List', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return apply_filters('gdlr_core_page_list_item_options', array(					
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'pages' => array(
								'title' => esc_html__('Select Pages', 'goodlayers-core'),
								'type' => 'multi-combobox',
								'options' => gdlr_core_get_post_list('page'),
								'description' => esc_html__('You can use Ctrl/Command button to select multiple items or remove the selected item. Leave this field blank to select all items in the list.', 'goodlayers-core'),
							),
							'orderby' => array(
								'title' => esc_html__('Order By', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'date' => esc_html__('Publish Date', 'goodlayers-core'), 
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
							)
						),
					),
					'settings' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'style' => array(
								'title' => esc_html__('Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'grid' => esc_html__('Grid', 'goodlayers-core'),
									'grid-no-space' => esc_html__('Grid No Space', 'goodlayers-core'),
									'thumbnail' => esc_html__('Thumbnail', 'goodlayers-core')
								)
							),
							'enable-title-divider' => array(
								'title' => esc_html__('Enable Title Divider',' goodlayers-core'),
								'type' => 'checkbox',
								'condition' => array( 'style' => array('grid', 'grid-no-space') ),
							),
							'divider-color' => array(
								'title' => esc_html__('Divider Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'default' => '#4c00ff',
								'condition' => array( 'style' => array('grid', 'grid-no-space'), 'enable-title-divider' => 'enable' ),
							),
							'divider-skewx' => array(
								'title' => esc_html__('Divider Skew X', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'condition' => array( 'style' => array('grid', 'grid-no-space'), 'enable-title-divider' => 'enable' ),
								'description' => esc_html__('Only input number here', 'goodlayers-core')
							),
							'overlay-color' => array(
								'title' => esc_html__('Background Overlay Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'default' => '#4c00ff',
								'condition' => array( 'style' => array('thumbnail') ),
							),
							'thumbnail-size' => array(
								'title' => esc_html__('Thumbnail Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size',
							),
							'enable-thumbnail-zoom-on-hover' => array(
								'title' => esc_html__('Thumbnail Zoom on Hover', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'enable' => esc_html__('Zoom', 'goodlayers-core'),
									'zoom-rotate' => esc_html__('Zoom & Rotate', 'goodlayers-core'),
									'disable' => esc_html__('Disable', 'goodlayers-core'),
								),
								'default' => 'enable',
							),
							'column-size' => array(
								'title' => esc_html__('Column Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 60 => 1, 30 => 2, 20 => 3, 15 => 4, 12 => 5 ),
								'default' => 20,
							),
							'layout' => array(
								'title' => esc_html__('Layout', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 
									'fitrows' => esc_html__('Fit Rows', 'goodlayers-core'),
									'carousel' => esc_html__('Carousel', 'goodlayers-core'),
									'masonry' => esc_html__('Masonry', 'goodlayers-core'),
								),
								'default' => 'fitrows',
							),
							'carousel-item-margin' => array(
								'title' => esc_html__('Carousel Item Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'layout' => 'carousel' )
							),
							'carousel-overflow' => array(
								'title' => esc_html__('Carousel Overflow', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Hidden', 'goodlayers-core'),
									'visible' => esc_html__('Visible', 'goodlayers-core')
								),
								'condition' => array( 'layout' => 'carousel' )
							),
							'carousel-scrolling-item-amount' => array(
								'title' => esc_html__('Carousel Scrolling Item Amount', 'goodlayers-core'),
								'type' => 'text',
								'default' => '1',
								'condition' => array( 'layout' => 'carousel' )
							),
							'carousel-autoslide' => array(
								'title' => esc_html__('Autoslide Carousel', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array( 'layout' => 'carousel' )
							),
							'carousel-navigation' => array(
								'title' => esc_html__('Carousel Navigation', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => (function_exists('gdlr_core_get_flexslider_navigation_types')? gdlr_core_get_flexslider_navigation_types(): array()),
								'default' => 'navigation',
								'condition' => array( 'layout' => 'carousel' )
							),
							'carousel-navigation-show-on-hover' => array(
								'title' => esc_html__('Carousel Navigation Display On Hover', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'carousel-navigation' => array('navigation-outer', 'navigation-inner') )
							),
							'carousel-navigation-align' => (function_exists('gdlr_core_get_flexslider_navigation_align')? gdlr_core_get_flexslider_navigation_align(): array()),
							'carousel-navigation-left-icon' => (function_exists('gdlr_core_get_flexslider_navigation_left_icon')? gdlr_core_get_flexslider_navigation_left_icon(): array()),
							'carousel-navigation-right-icon' => (function_exists('gdlr_core_get_flexslider_navigation_right_icon')? gdlr_core_get_flexslider_navigation_right_icon(): array()),
							'carousel-navigation-icon-color' => (function_exists('gdlr_core_get_flexslider_navigation_icon_color')? gdlr_core_get_flexslider_navigation_icon_color(): array()),
							'carousel-navigation-icon-hover-color' => (function_exists('gdlr_core_get_flexslider_navigation_icon_hover_color')? gdlr_core_get_flexslider_navigation_icon_hover_color(): array()),
							'carousel-navigation-icon-bg' => (function_exists('gdlr_core_get_flexslider_navigation_icon_background')? gdlr_core_get_flexslider_navigation_icon_background(): array()),
							'carousel-navigation-icon-padding' => (function_exists('gdlr_core_get_flexslider_navigation_icon_padding')? gdlr_core_get_flexslider_navigation_icon_padding(): array()),
							'carousel-navigation-icon-radius' => (function_exists('gdlr_core_get_flexslider_navigation_icon_radius')? gdlr_core_get_flexslider_navigation_icon_radius(): array()),
							'carousel-navigation-size' => (function_exists('gdlr_core_get_flexslider_navigation_icon_size')? gdlr_core_get_flexslider_navigation_icon_size(): array()),
							'carousel-navigation-margin' => (function_exists('gdlr_core_get_flexslider_navigation_margin')? gdlr_core_get_flexslider_navigation_margin(): array()),
							'carousel-navigation-side-margin' => (function_exists('gdlr_core_get_flexslider_navigation_side_margin')? gdlr_core_get_flexslider_navigation_side_margin(): array()),
							'carousel-navigation-icon-margin' => (function_exists('gdlr_core_get_flexslider_navigation_icon_margin')? gdlr_core_get_flexslider_navigation_icon_margin(): array()),
							'carousel-bullet-style' => array(
								'title' => esc_html__('Carousel Bullet Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => (function_exists('gdlr_core_get_flexslider_bullet_itypes')? gdlr_core_get_flexslider_bullet_itypes(): array()),
								'condition' => array( 'layout' => 'carousel', 'carousel-navigation' => array('bullet','both') ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'carousel-bullet-top-margin' => array(
								'title' => esc_html__('Carousel Bullet Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'layout' => 'carousel', 'carousel-navigation' => array('bullet','both') )
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
							'title-font-weight' => array(
								'title' => esc_html__('Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'title-font-style' => array(
								'title' => esc_html__('Title Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Default', 'goodlayers-core'),
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
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
						),
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						),
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						),
					)
				));
			}

			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-page-list-<?php echo esc_attr($id); ?>" >
if( document.readyState == 'complete' ){
	jQuery(document).ready(function(){
		var item_preview = jQuery('#gdlr-core-preview-page-list-<?php echo esc_attr($id); ?>').parent();
		item_preview.gdlr_core_flexslider().gdlr_core_isotope().gdlr_core_content_script();
	});
}else{
	jQuery(window).on('load', function(){
		setTimeout(function(){
			var item_preview = jQuery('#gdlr-core-preview-page-list-<?php echo esc_attr($id); ?>').parent();
			item_preview.gdlr_core_flexslider().gdlr_core_isotope().gdlr_core_content_script();
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
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				$settings['style'] = empty($settings['style'])? 'grid': $settings['style'];
				$settings['column-size'] = empty($settings['column-size'])? 20: $settings['column-size'];
				$settings['thumbnail-size'] = empty($settings['thumbnail-size'])? 'full': $settings['thumbnail-size'];
				$settings['enable-title-divider'] = empty($settings['enable-title-divider'])? 'enable': $settings['enable-title-divider'];

				$settings['no-space'] = false;
				if( in_array($settings['style'], array('grid-no-space')) ){
					$settings['no-space'] = true;
					$settings['style'] = str_replace('-no-space', '', $settings['style']);
				}

 				$custom_style  = '';
				if( !empty($settings['carousel-navigation-icon-hover-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-flexslider-custom-nav i:hover{ color: ' . $settings['carousel-navigation-icon-hover-color'] . ' !important; }';
				}
				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_page_list_id;

						if( $preview ){
							$gdlr_core_page_list_id = empty($gdlr_core_page_list_id)? array(): $gdlr_core_page_list_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_item_id = mt_rand(0, 99999);
							while( in_array($rnd_item_id, $gdlr_core_page_list_id) ){
								$rnd_item_id = mt_rand(0, 99999);
							}
							$gdlr_core_page_list_id[] = $rnd_item_id;
							$settings['id'] = 'gdlr-core-page-list-' . $rnd_item_id;
						}else{
							$gdlr_core_page_list_id = empty($gdlr_core_page_list_id)? 1: $gdlr_core_page_list_id + 1;
							$settings['id'] = 'gdlr-core-page-list-' . $gdlr_core_page_list_id;
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

				$extra_class = ($settings['no-space'])? 'gdlr-core-item-pdlr': '';

				// start printing item
				$ret  = '<div class="gdlr-core-page-list-item gdlr-core-item-pdb clearfix ' . $extra_class . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				$query = self::get_query($settings);

				if( $settings['layout'] == 'carousel' ){
					$slides = array();
					$column_no = 60 / intval($settings['column-size']);

					$flex_atts = array(
						'carousel' => true,
						'margin' => empty($settings['carousel-item-margin'])? '': $settings['carousel-item-margin'],
						'overflow' => empty($settings['carousel-overflow'])? '': $settings['carousel-overflow'],
						'column' => $column_no,
						'move' => empty($settings['carousel-scrolling-item-amount'])? '': $settings['carousel-scrolling-item-amount'],
						'navigation' => empty($settings['carousel-navigation'])? 'navigation': $settings['carousel-navigation'],
						'navigation-on-hover' => empty($settings['carousel-navigation-show-on-hover'])? 'disable': $settings['carousel-navigation-show-on-hover'],
						'navigation-align' => empty($settings['carousel-navigation-align'])? '': $settings['carousel-navigation-align'],
						'navigation-size' => empty($settings['carousel-navigation-size'])? '': $settings['carousel-navigation-size'],
						'navigation-icon-color' => empty($settings['carousel-navigation-icon-color'])? '': $settings['carousel-navigation-icon-color'],
						'navigation-icon-background' => empty($settings['carousel-navigation-icon-bg'])? '': $settings['carousel-navigation-icon-bg'],
						'navigation-icon-padding' => empty($settings['carousel-navigation-icon-padding'])? '': $settings['carousel-navigation-icon-padding'],
						'navigation-icon-radius' => empty($settings['carousel-navigation-icon-radius'])? '': $settings['carousel-navigation-icon-radius'],
						'navigation-margin' => empty($settings['carousel-navigation-margin'])? '': $settings['carousel-navigation-margin'],
						'navigation-side-margin' => empty($settings['carousel-navigation-side-margin'])? '': $settings['carousel-navigation-side-margin'],
						'navigation-icon-margin' => empty($settings['carousel-navigation-icon-margin'])? '': $settings['carousel-navigation-icon-margin'],
						'navigation-left-icon' => empty($settings['carousel-navigation-left-icon'])? '': $settings['carousel-navigation-left-icon'],
						'navigation-right-icon' => empty($settings['carousel-navigation-right-icon'])? '': $settings['carousel-navigation-right-icon'],
						'bullet-style' => empty($settings['carousel-bullet-style'])? '': $settings['carousel-bullet-style'],
						'controls-top-margin' => empty($settings['carousel-bullet-top-margin'])? '': $settings['carousel-bullet-top-margin'],
						'nav-parent' => 'gdlr-core-page-list-item',
						'disable-autoslide' => (empty($settings['carousel-autoslide']) || $settings['carousel-autoslide'] == 'enable')? '': true,
						'mglr' => ($settings['no-space'] == 'yes'? false: true)
					);
					
					gdlr_core_setup_admin_postdata();

					while($query->have_posts()){ $query->the_post();
						$slides[] = self::get_content_html($settings);
					} // while

					$ret .= gdlr_core_get_flexslider($slides, $flex_atts);

					wp_reset_postdata();
					gdlr_core_reset_admin_postdata();
					
				// fitrows style
				}else{
					
					gdlr_core_setup_admin_postdata();

					$ret .= '<div class="gdlr-core-page-list-item-holder gdlr-core-js-2 clearfix" data-layout="' . $settings['layout'] . '" >';
					
					$column_sum = 0;
					while($query->have_posts()){ $query->the_post();
						$additional_class  = $settings['no-space']? '': 'gdlr-core-item-pdlr';
						$additional_class .= empty($settings['column-size'])? '': ' gdlr-core-column-' . $settings['column-size'];

						if( $column_sum == 0 || $column_sum + intval($settings['column-size']) > 60 ){
							$column_sum = intval($settings['column-size']);
							$additional_class .= ' gdlr-core-column-first';
						}else{
							$column_sum += intval($settings['column-size']);
						}

						$ret .= '<div class="gdlr-core-item-list ' . esc_attr($additional_class) . '" >';
						$ret .= self::get_content_html($settings);
						$ret .= '</div>';
					}
					$ret .= '</div>';

					wp_reset_postdata();
					gdlr_core_reset_admin_postdata();
					
				}
				
				$ret .= '</div>'; // gdlr-core-page-list-item
				$ret .= $custom_style;
				
				return $ret;
			}
			
			// query the page
			static function get_query( $settings ){
				
				$args = array( 
					'post_type' => 'page', 
					'post_status' => 'publish', 
					'suppress_filters' => false,
					'post__in' => empty($settings['pages'])? array(): $settings['pages']
				);
				
				// variable
				$args['orderby'] = $settings['orderby'];
				$args['order'] = $settings['order'];
				
				$query = new WP_Query( $args );

				return $query;
			}

			// get content
			static function get_content_html($settings){

				switch($settings['style']){
					case 'grid':
						return self::get_grid_html($settings);
					case 'thumbnail':
						return self::get_modern_html($settings);
				}

				return '';
			}

			static function get_title_html($settings){

				return '<h3 class="gdlr-core-title gdlr-core-skin-title" ' . gdlr_core_esc_style(array(
					'font-size' => empty($settings['title-font-size'])? '': $settings['title-font-size'],
					'font-weight' => empty($settings['title-font-weight'])? '': $settings['title-font-weight'],
					'font-style' => empty($settings['title-font-style'])? '': $settings['title-font-style'],
					'letter-spacing' => empty($settings['title-letter-spacing'])? '': $settings['title-letter-spacing'],
					'text-transform' => (empty($settings['title-text-transform']) || $settings['title-text-transform'] == 'none')? '': $settings['title-text-transform']
				)) . ' ><a href="' . get_permalink() . '" ' . gdlr_core_esc_style(array(
					'color' => empty($settings['title-color'])? '': $settings['title-color']
				)) . ' >' . get_the_title() . '</a></h3>';

			}

			static function get_grid_html($settings){
				$extra_class = empty($settings['divider-skewx'])? '': 'gdlr-core-outer-frame-element';
				$ret  = '<div class="gdlr-core-page-list gdlr-core-style-grid " >';

				$feature_image = get_post_thumbnail_id();
				if( !empty($feature_image) ){
					$ret .= '<div class="gdlr-core-thumbnail gdlr-core-media-image ' . esc_attr($extra_class);
					if( empty($settings['enable-thumbnail-zoom-on-hover']) || $settings['enable-thumbnail-zoom-on-hover'] == 'enable' ){
						$ret .= ' gdlr-core-zoom-on-hover';
					}else if( $settings['enable-thumbnail-zoom-on-hover'] == 'zoom-rotate' ){
						$ret .= ' gdlr-core-zoom-rotate-on-hover';
					}
					$ret .= '" ><a href="' . get_permalink() . '" >';
					$ret .= gdlr_core_get_image($feature_image, $settings['thumbnail-size']);
					$ret .= '</a></div>';
				}
 
				$ret .= self::get_title_html($settings);

				if( !empty($settings['enable-title-divider']) && $settings['enable-title-divider'] == 'enable' ){
					$ret .= '<div class="gdlr-core-title-divider" ' . gdlr_core_esc_style(array(
						'border-color' => empty($settings['divider-color'])? '': $settings['divider-color'],
						'skewx' => empty($settings['divider-skewx'])? '': $settings['divider-skewx'] 
					)) . ' ></div>';
				}
				$ret .= '</div>';

				return $ret;
			}

			static function get_modern_html($settings){
				$extra_class = empty($settings['divider-skewx'])? '': 'gdlr-core-outer-frame-element';
				$ret  = '<div class="gdlr-core-page-list gdlr-core-style-modern gdlr-core-item-mgb ' . esc_attr($extra_class);
				if( empty($settings['enable-thumbnail-zoom-on-hover']) || $settings['enable-thumbnail-zoom-on-hover'] == 'enable' ){
					$ret .= ' gdlr-core-zoom-on-hover';
				}else if( $settings['enable-thumbnail-zoom-on-hover'] == 'zoom-rotate' ){
					$ret .= ' gdlr-core-zoom-rotate-on-hover';
				}
				$ret .= '" >';

				$feature_image = get_post_thumbnail_id();
				if( !empty($feature_image) ){
					$ret .= '<div class="gdlr-core-thumbnail gdlr-core-media-image" ><a href="' . get_permalink() . '" >';
					$ret .= gdlr_core_get_image($feature_image, $settings['thumbnail-size']);
					$ret .= '</a></div>';
				}
				
				$ret .= '<div class="gdlr-core-thumbnail-overlay" ' . gdlr_core_esc_style(array(
					'background-color' => empty($settings['overlay-color'])? '': $settings['overlay-color']
				)) . ' ></div>';
				$ret .= '<div class="gdlr-core-thumbnail-overlay-content" >';
				$ret .= self::get_title_html($settings);
				$ret .= '</div>';

				$ret .= '</div>';

				return $ret;
			}
			
		} // gdlr_core_pb_element_page_list
	} // class_exists	