<?php
	/*	
	*	Goodlayers Product Item
	*/
	
	if( !class_exists('gdlr_core_product_item') ){
		class gdlr_core_product_item{
			
			var $settings = '';
			
			// init the variable
			function __construct( $settings = array() ){
				
				$this->settings = wp_parse_args($settings, array(
					'category' => '', 
					'tag' => '', 
					'stock-status' => 'all', 
					'num-fetch' => '9', 
					'layout' => 'fitrows',
					'thumbnail-size' => 'full', 
					'orderby' => 'date', 
					'order' => 'desc',
					'product-style' => 'grid', 
					'has-column' => 'no',
					'no-space' => 'no',
					'column-size' => 20,
					'pagination' => 'none'
				));

				if( $this->settings['orderby'] == 'rand' ){
					$this->settings['orderby'] = 'RAND(' . rand(1, 1000) . ')';
				} 
			}
			
			// get the content of the product item
			function get_content(){

				remove_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_price', 10 );
				remove_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_rating', 5 );
				
				if( !empty($this->settings['column-size']) ){
					gdlr_core_set_container_multiplier(intval($this->settings['column-size']) / 60, false);
				}

				$ret = '';
				$filter_cats = array();
				if( empty($this->settings['query']) ){
					if( $this->settings['layout'] != 'carousel' ){
						if( !empty($this->settings['enable-category-filter']) && $this->settings['enable-category-filter'] == 'enable' ){
							$filter_cats = $this->settings['category'];
							
							if( !empty($this->settings['category']) && sizeof($this->settings['category']) > 1 ){
								$this->settings['category'] = array($this->settings['category'][0]);
							}
						}
					}

					$query = $this->get_product_query();
				}else{
					$query = $this->settings['query'];
				}
				
				// filterer
				if( !empty($this->settings['enable-category-filter']) ){
					if( $this->settings['enable-category-filter'] == 'enable' ){
						$ret .= $this->get_ajax_filterer($filter_cats);
					}else if( $this->settings['enable-category-filter'] == 'text' ){
						$extra_class  = 'gdlr-core-style-text ';
						if( $this->settings['no-space'] == 'no' && $this->settings['layout'] != 'carousel' ){
							$extra_class .= 'gdlr-core-item-pdlr ';
						}
						$extra_class .= empty($this->settings['filterer-align'])? '': ' gdlr-core-' . $this->settings['filterer-align'] . '-align';
						$extra_class .= apply_filters('gdlr_core_blog_filterer_class', '', $this->settings);
						
						$ret .= gdlr_core_get_ajax_filterer('product', 'product_cat', $this->settings, 'gdlr-core-product-item-holder', $extra_class);
					}else if( $this->settings['enable-category-filter'] == 'dropdown' ){
						$ret .= $this->get_dropdown_ajax_filterer($filter_cats);
					}
				}

				// carousel style
				if( $this->settings['layout'] == 'carousel' ){
					$slides = array();
					$column_no = 60 / intval($this->settings['column-size']);

					$flex_atts = array(
						'carousel' => true,
						'overflow' => empty($this->settings['carousel-overflow'])? '': $this->settings['carousel-overflow'],
						'column' => $column_no,
						'move' => empty($this->settings['carousel-scrolling-item-amount'])? '': $this->settings['carousel-scrolling-item-amount'],
						'navigation' => empty($this->settings['carousel-navigation'])? 'navigation': $this->settings['carousel-navigation'],
						'navigation-on-hover' => empty($this->settings['carousel-navigation-show-on-hover'])? 'disable': $this->settings['carousel-navigation-show-on-hover'],
						'navigation-align' => empty($this->settings['carousel-navigation-align'])? '': $this->settings['carousel-navigation-align'],
						'navigation-size' => empty($this->settings['carousel-navigation-size'])? '': $this->settings['carousel-navigation-size'],
						'navigation-icon-color' => empty($this->settings['carousel-navigation-icon-color'])? '': $this->settings['carousel-navigation-icon-color'],
						'navigation-icon-background' => empty($this->settings['carousel-navigation-icon-bg'])? '': $this->settings['carousel-navigation-icon-bg'],
						'navigation-icon-padding' => empty($this->settings['carousel-navigation-icon-padding'])? '': $this->settings['carousel-navigation-icon-padding'],
						'navigation-icon-radius' => empty($this->settings['carousel-navigation-icon-radius'])? '': $this->settings['carousel-navigation-icon-radius'],
						'navigation-margin' => empty($this->settings['carousel-navigation-margin'])? '': $this->settings['carousel-navigation-margin'],
						'navigation-icon-margin' => empty($this->settings['carousel-navigation-icon-margin'])? '': $this->settings['carousel-navigation-icon-margin'],
						'navigation-left-icon' => empty($this->settings['carousel-navigation-left-icon'])? '': $this->settings['carousel-navigation-left-icon'],
						'navigation-right-icon' => empty($this->settings['carousel-navigation-right-icon'])? '': $this->settings['carousel-navigation-right-icon'],
						'bullet-style' => empty($this->settings['carousel-bullet-style'])? '': $this->settings['carousel-bullet-style'],
						'controls-top-margin' => empty($this->settings['carousel-bullet-top-margin'])? '': $this->settings['carousel-bullet-top-margin'],
						'nav-parent' => 'gdlr-core-product-item',
						'disable-autoslide' => (empty($this->settings['carousel-autoslide']) || $this->settings['carousel-autoslide'] == 'enable')? '': true,
						'mglr' => ($this->settings['no-space'] == 'yes'? false: true)
					);

					gdlr_core_setup_admin_postdata();

					$product_style = new gdlr_core_product_style();
					if( function_exists('woocommerce_product_loop_start') ){
						woocommerce_product_loop_start(false);
					}
					while($query->have_posts()){ $query->the_post();
						$slides[] = $product_style->get_content( $this->settings );
					} // while
					if( function_exists('woocommerce_product_loop_end') ){
						woocommerce_product_loop_end(false);
					}

					$ret .= '<div class="gdlr-core-product-item-holder gdlr-core-js-2 clearfix" data-layout="' . $this->settings['layout'] . '" >';
					$ret .= gdlr_core_get_flexslider($slides, $flex_atts);
					$ret .= '</div>';

					wp_reset_postdata();
					gdlr_core_reset_admin_postdata();
					
				// fitrows style
				}else{

					gdlr_core_setup_admin_postdata();
					
					$ret .= '<div class="gdlr-core-product-item-holder gdlr-core-js-2 clearfix" data-layout="' . $this->settings['layout'] . '" >';
					$ret .= $this->get_product_grid_content($query);
					$ret .= '</div>';

					wp_reset_postdata();
					gdlr_core_reset_admin_postdata();

					// pagination
					if( $this->settings['pagination'] != 'none' ){
						$extra_class = '';
						if( $this->settings['no-space'] == 'no' ){
							$extra_class = 'gdlr-core-item-pdlr';
						}

						if( $this->settings['pagination'] == 'page' ){
							if( !empty($this->settings['enable-category-filter']) && $this->settings['enable-category-filter'] == 'enable' ){
								$ret .= gdlr_core_get_ajax_pagination('product', $this->settings, $query->max_num_pages, 'gdlr-core-product-item-holder', $extra_class);
							}else{
								$ret .= gdlr_core_get_pagination($query->max_num_pages, $this->settings, $extra_class);
							}
						}else if( $this->settings['pagination'] == 'load-more' ){
							$paged = empty($query->query['paged'])? 2: intval($query->query['paged']) + 1;
							$ret .= gdlr_core_get_ajax_load_more('product', $this->settings, $paged, $query->max_num_pages, 'gdlr-core-product-item-holder', $extra_class);
						}
					}
				}

				gdlr_core_set_container_multiplier(1, false);

				add_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_price', 10 );
				add_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_rating', 5 );
				

				return $ret;
			}

			function get_ajax_filterer($filter_cats){
				
				$extra_class = '';
				if( $this->settings['no-space'] == 'no' && $this->settings['layout'] != 'carousel' ){
					$extra_class = 'gdlr-core-item-pdlr';
				}
				$extra_class .= empty($this->settings['filterer-align'])? '': ' gdlr-core-' . $this->settings['filterer-align'] . '-align';
							

				$ret  = '<div class="gdlr-core-filterer-wrap gdlr-core-style-box gdlr-core-js ' . esc_attr($extra_class) . '" ';
				$ret .= 'data-ajax="gdlr_core_product_ajax" ';
				$ret .= 'data-settings="' . esc_attr(json_encode($this->settings)) . '" ';
				$ret .= 'data-target="gdlr-core-product-item-holder" ';
				$ret .= 'data-target-action="replace" >';
				
				$term_atts = array(
					'taxonomy' => 'product_cat', 
					'hide_empty' => 0,
					'number' => 999
				);
				$has_active = false;

				// for all
				if( empty($filter_cats) ){
					$has_active = true;
					$ret .= '<a href="#" class="gdlr-core-filterer gdlr-core-active" >';
					if( !empty($this->settings['filterer-all-icon']) ){
						$ret .= '<span class="gdlr-core-icon" ><i class="' .  esc_attr($this->settings['filterer-all-icon']) . '" ></i></span>';
					}
					$ret .= esc_html__('All', 'goodlayers-core');
					$ret .= '</a>';

				// parent category
				}else if( sizeof($filter_cats) == 1 ){
					$has_active = true;
					$term = get_term_by('slug', $filter_cats[0], 'product_cat');
					$ret .= '<a href="#" class="gdlr-core-filterer gdlr-core-active" >';
					$ret .= gdlr_core_escape_content($term->name);
					$ret .= '</a>';
					$term_atts['parent'] = $term->term_id;
	
				// multiple category select
				}else{
					$term_atts['slug'] = $filter_cats;
				}

				$term_list = get_categories($term_atts);
				if( !empty($term_list) ){
					foreach($term_list as $term){
						if( empty($filter_cats) && $term->slug == 'uncategorized' ){ continue; }

						$icon = get_term_meta($term->term_id, 'icon', true);

						$ret .= '<a href="#" class="gdlr-core-filterer ' . ($has_active? '': 'gdlr-core-active') . '" data-ajax-name="category" data-ajax-value="' . esc_attr($term->slug) . '" >';
						if( !empty($icon) ){
							$ret .= '<span class="gdlr-core-icon" ><i class="' .  esc_attr($icon) . '" ></i></span>';
						}
						$ret .= '<span class="gdlr-core-head" >' . gdlr_core_escape_content($term->name) . '</span>';
						$ret .= '</a>';

						$has_active = true;
					}
				}
				
				$ret .= '</div>'; // gdlr-core-filterer-wrap
	
				return $ret;
			}

			function get_dropdown_ajax_filterer($filter_cats){
				
				$extra_class = '';
				if( $this->settings['no-space'] == 'no' && $this->settings['layout'] != 'carousel' ){
					$extra_class = 'gdlr-core-item-pdlr';
				}
				$extra_class .= empty($this->settings['filterer-align'])? '': ' gdlr-core-' . $this->settings['filterer-align'] . '-align';		

				$ret  = '<div class="gdlr-core-filterer-wrap gdlr-core-style-dropdown gdlr-core-js ' . esc_attr($extra_class) . '" ';
				$ret .= 'data-ajax="gdlr_core_product_ajax" ';
				$ret .= 'data-settings="' . esc_attr(json_encode($this->settings)) . '" ';
				$ret .= 'data-target="gdlr-core-product-item-holder" ';
				$ret .= 'data-target-action="replace" >';

				$term_atts = array(
					'taxonomy' => 'product_cat', 
					'hide_empty' => 0,
					'number' => 999
				);
				$has_active = false;
				$active_text = '';

				// for all
				$filter_list = '';
				if( empty($filter_cats) ){
					$has_active = true;
					$active_text = esc_html__('Browser Categories', 'goodlayers-core');
					$filter_list .= '<a href="#" class="gdlr-core-filterer gdlr-core-active" >';
					$filter_list .= esc_html__('All', 'goodlayers-core');
					$filter_list .= '</a>';

				// parent category
				}else if( sizeof($filter_cats) == 1 ){
					$has_active = true;
					$term = get_term_by('slug', $filter_cats[0], 'product_cat');
					$active_text = gdlr_core_escape_content($term->name);
					$filter_list .= '<a href="#" class="gdlr-core-filterer gdlr-core-active" >';
					$filter_list .= gdlr_core_escape_content($term->name);
					$filter_list .= '</a>';
					$term_atts['parent'] = $term->term_id;
	
				// multiple category select
				}else{
					$term_atts['slug'] = $filter_cats;
				}

				$term_list = get_categories($term_atts);
				if( !empty($term_list) ){
					foreach($term_list as $term){
						if( empty($filter_cats) && $term->slug == 'uncategorized' ){ continue; }
						if( !$has_active ){ $active_text = gdlr_core_escape_content($term->name); }

						$filter_list .= '<a href="#" class="gdlr-core-filterer ' . ($has_active? '': 'gdlr-core-active') . '" data-ajax-name="category" data-ajax-value="' . esc_attr($term->slug) . '" >';
						$filter_list .= '<span class="gdlr-core-head" >' . gdlr_core_escape_content($term->name) . '</span>';
						$filter_list .= '</a>';

						$has_active = true;
					}
				}
				$ret .= '<div class="gdlr-core-filterer-dropdown-content" >';
				$ret .= '<div class="gdlr-core-filterer-dropdown-text" >' . $active_text . '</div>';
				$ret .= '<div class="gdlr-core-filterer-dropdown" >' . $filter_list . '</div>';
				$ret .= '</div>';
				$ret .= '</div>'; // gdlr-core-filterer-wrap
	
				return $ret;
			}

			// get content of non carousel product item
			function get_product_grid_content( $query ){

				$ret = '';
				$column_sum = 0;
				$product_style = new gdlr_core_product_style();

				if( function_exists('woocommerce_product_loop_start') ){
					$ret .= woocommerce_product_loop_start(false);
				}
				if( is_archive() ){
					ob_start();
					woocommerce_product_subcategories();
					$sub_cat_content = ob_get_contents();
					ob_end_clean();

					if( !empty($sub_cat_content) ){
						echo '<ul class="gdlr-core-product-sub-category clearfix" >' . $sub_cat_content . '</ul>';
					}
				}
				while($query->have_posts()){ $query->the_post();

					$args = $this->settings;

					if( $this->settings['has-column'] == 'yes' ){
						$additional_class  = ($this->settings['no-space'] == 'yes')? '': ' gdlr-core-item-pdlr';
						$additional_class .= empty($this->settings['column-size'])? '': ' gdlr-core-column-' . $this->settings['column-size'];

						if( $column_sum == 0 || $column_sum + intval($this->settings['column-size']) > 60 ){
							$column_sum = intval($this->settings['column-size']);
							$additional_class .= ' gdlr-core-column-first';
						}else{
							$column_sum += intval($this->settings['column-size']);
						}

						$ret .= '<div class="gdlr-core-item-list ' . esc_attr($additional_class) . '" >';
					}

					$ret .= $product_style->get_content($args);

					if( $this->settings['has-column'] == 'yes' ){
						$ret .= '</div>';
					}
				} // while
				if( function_exists('woocommerce_product_loop_end') ){
					$ret .= woocommerce_product_loop_end(false);
				}

				return $ret;
			}
			
			// query the post
			function get_product_query(){
				
				$args = array( 'post_type' => 'product', 'post_status' => 'publish', 'suppress_filters' => false );
				
				// category - tag selection
				if( !empty($this->settings['category']) || !empty($this->settings['tag']) ){
					$args['tax_query'] = array(
						'relation' => empty($this->settings['relation'])? 'OR': $this->settings['relation']
					);
					
					if( !empty($this->settings['category']) ){
						array_push($args['tax_query'], array('terms'=>$this->settings['category'], 'taxonomy'=>'product_cat', 'field'=>'slug'));
					}
					if( !empty($this->settings['tag']) ){
						array_push($args['tax_query'], array('terms'=>$this->settings['tag'], 'taxonomy'=>'product_tag', 'field'=>'slug'));
					}
				}

				// pagination
				if( empty($this->settings['paged']) ){
					$args['paged'] = (get_query_var('paged'))? get_query_var('paged') : get_query_var('page');
					$args['paged'] = empty($args['paged'])? 1: $args['paged'];
				}else{
					$args['paged'] = $this->settings['paged'];
				}
				$this->settings['paged'] = $args['paged'];				

				if( !empty($this->settings['stock-status']) && $this->settings['stock-status'] != 'all' ){
					 $args['meta_query'] = array(
				        array(
				            'key' => '_stock_status',
				            'value' => $this->settings['stock-status']
				        )
				    );
				}

				// variable
				$args['posts_per_page'] = $this->settings['num-fetch'];
				$args['orderby'] = $this->settings['orderby'];
				$args['order'] = $this->settings['order'];
				
				$query = new WP_Query( $args );

				return $query;
			}
			
		} // gdlr_core_product_item
	} // class_exists
	
	add_action('wp_ajax_gdlr_core_product_ajax', 'gdlr_core_product_ajax');
	add_action('wp_ajax_nopriv_gdlr_core_product_ajax', 'gdlr_core_product_ajax');
	if( !function_exists('gdlr_core_product_ajax') ){
		function gdlr_core_product_ajax(){

			if( !empty($_POST['settings']) ){

				$settings = $_POST['settings'];
				if( !empty($_POST['option']) ){	
					$settings[$_POST['option']['name']] = $_POST['option']['value'];

					if( $_POST['option']['name'] == 'category' ){
						$settings['paged'] = 1;
					}
				}else{
					$settings['paged'] = 1;
				}

				$product_item = new gdlr_core_product_item($settings);
				$query = $product_item->get_product_query();	

				remove_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_price', 10 );
				remove_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_rating', 5 );

				if( $settings['layout'] == 'carousel' ){
					
					$slides = array();
					$column_no = 60 / intval($settings['column-size']);

					$flex_atts = array(
						'carousel' => true,
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
						'navigation-icon-margin' => empty($settings['carousel-navigation-icon-margin'])? '': $settings['carousel-navigation-icon-margin'],
						'navigation-left-icon' => empty($settings['carousel-navigation-left-icon'])? '': $settings['carousel-navigation-left-icon'],
						'navigation-right-icon' => empty($settings['carousel-navigation-right-icon'])? '': $settings['carousel-navigation-right-icon'],
						'bullet-style' => empty($settings['carousel-bullet-style'])? '': $settings['carousel-bullet-style'],
						'controls-top-margin' => empty($settings['carousel-bullet-top-margin'])? '': $settings['carousel-bullet-top-margin'],
						'nav-parent' => 'gdlr-core-product-item',
						'disable-autoslide' => (empty($settings['carousel-autoslide']) || $settings['carousel-autoslide'] == 'enable')? '': true,
						'mglr' => ($settings['no-space'] == 'yes'? false: true)
					);

					$product_style = new gdlr_core_product_style();
					if( function_exists('woocommerce_product_loop_start') ){
						woocommerce_product_loop_start(false);
					}
					while($query->have_posts()){ $query->the_post();
						$slides[] = $product_style->get_content( $settings );
					} // while
					if( function_exists('woocommerce_product_loop_end') ){
						woocommerce_product_loop_end(false);
					}

					$ret = array(
						'status'=> 'success',
						'content'=> gdlr_core_get_flexslider($slides, $flex_atts)
					);

				}else{
					$ret = array(
						'status'=> 'success',
						'content'=> $product_item->get_product_grid_content($query)
					);

					if( !empty($settings['pagination']) && $settings['pagination'] != 'none' ){
						if( $settings['pagination'] == 'load-more' ){
							$paged = empty($query->query['paged'])? 2: intval($query->query['paged']) + 1;
							$extra_class = ($settings['no-space'] == 'yes')? '': 'gdlr-core-item-pdlr';
	
							$ret['load_more'] = gdlr_core_get_ajax_load_more('product', $settings, $paged, $query->max_num_pages, 'gdlr-core-product-item-holder', $extra_class);
							$ret['load_more'] = empty($ret['load_more'])? 'none': $ret['load_more'];
						}else if( empty($_POST['option']['name']) || $_POST['option']['name'] == 'category' ){
							$ret['pagination'] = gdlr_core_get_ajax_pagination('product', $settings, $query->max_num_pages, 'gdlr-core-product-item-holder', $extra_class);
							$ret['pagination'] = empty($ret['pagination'])? 'none': $ret['pagination'];
						}
					} 
				}

				add_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_price', 10 );
				add_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_rating', 5 );

				die(json_encode($ret));
			}else{
				die(json_encode(array(
					'status'=> 'failed',
					'message'=> esc_html__('Settings variable is not defined.', 'goodlayers-core')
				)));
			}

		} // gdlr_core_product_load_more
	} // function_exists

	